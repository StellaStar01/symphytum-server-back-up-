# symphytum-server

PS for hololive Dreams (very, very work in progress)

start: `./bin rpc -r`

needs https://github.com/yuvlian/symphytum with this config:

```
[symphytum]
enable_patches=true

redirect_game_requests=true

redirect_asset_requests=false

disable_encryption=false

disable_cert_pinning=true

use_custom_root_cert=false

game_server=https://127.0.0.1:3000/

asset_server=https://127.0.0.1:3000/

custom_root_cert=

log_level=2
```

if u just want protos: https://github.com/yuvlian/hololive-Dreams-Proto
