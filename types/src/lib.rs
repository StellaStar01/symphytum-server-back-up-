#![allow(unused_comparisons)]

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

// Include generated validation implementations
include!(concat!(env!("OUT_DIR"), "/validate_impl.rs"));
