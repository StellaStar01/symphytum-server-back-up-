#![allow(unused_comparisons)]

pub use prost_protovalidate::Validate;

pub mod reflection;

pub mod common {
    tonic::include_proto!("common");
}

pub mod dto {
    pub mod multi_game_result {
        tonic::include_proto!("dto.multi_game_result");
    }
}

pub mod entity {
    pub mod master {
        tonic::include_proto!("entity.master");
    }
    pub mod transaction {
        tonic::include_proto!("entity.transaction");
    }
}

pub mod options {
    pub mod check_option {
        tonic::include_proto!("options.check_option");
    }
    pub mod multi_game_result {
        tonic::include_proto!("options.multi_game_result");
    }
}

pub mod rpc {
    pub mod api {
        tonic::include_proto!("rpc.api");
        pub mod common {
            tonic::include_proto!("rpc.api.common");
        }
    }
    pub mod multi_game_server_manager {
        tonic::include_proto!("rpc.multi_game_server_manager");
    }
}

pub mod enums {
    tonic::include_proto!("enums");
}

pub mod buf {
    pub mod validate {
        tonic::include_proto!("buf.validate");
    }
}

pub mod google {
    pub mod api {
        tonic::include_proto!("google.api");
    }
}

include!(concat!(env!("OUT_DIR"), "/validate_impl.rs"));

#[cfg(test)]
mod tests {
    use prost_reflect::Value;

    use crate::reflection::*;

    #[test]
    fn formats_scalars_and_enum_names() {
        let desc = DESCRIPTOR_POOL
            .get_message_by_name("common.Consumption")
            .unwrap();
        let mut msg = DynamicMessage::new(desc);
        msg.set_field_by_name("resource_type", Value::EnumNumber(2)); // RESOURCE_TYPE_CARD
        msg.set_field_by_name("resource_id", Value::String("card_1".into()));
        msg.set_field_by_name("quantity", Value::I64(5));
        assert_eq!(
            format_message(&msg),
            "Consumption {\n    resource_type: RESOURCE_TYPE_CARD,\n    resource_id: \"card_1\",\n    quantity: 5,\n}"
        );
    }

    #[test]
    fn keeps_defaults_and_nests_messages() {
        let desc = DESCRIPTOR_POOL
            .get_message_by_name("rpc.api.SystemGetSystemInfoResponse")
            .unwrap();
        let mut msg = DynamicMessage::new(desc);
        let review_desc = DESCRIPTOR_POOL
            .get_message_by_name("rpc.api.SystemGetSystemInfoResponse.ReviewInfo")
            .unwrap();
        let mut review = DynamicMessage::new(review_desc);
        review.set_field_by_name("is_in_review", Value::Bool(true));
        msg.set_field_by_name("review_info", Value::Message(review));
        let out = format_message(&msg);
        assert!(out.starts_with("SystemGetSystemInfoResponse {\n"));
        assert!(out.contains("    review_info: Some(ReviewInfo {\n"));
        assert!(out.contains("        is_in_review: true,\n"));
        assert!(out.contains("        api_host_in_review: \"\",\n"));
        assert!(out.contains("    maintenance_info: None,\n"));
        assert!(out.contains("    title_download_gacha_asset_infos: [],\n"));
    }
}
