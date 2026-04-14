# Nginx — combined access log

## Description
Collecte les access logs nginx (format combined).
Normalise en OCSF classe 4001 (Network Activity).

## Format attendu
```
10.0.0.1 - user [14/Apr/2024:10:23:45 +0000] "GET /path HTTP/1.1" 200 1234 "referer" "user-agent"
```

## Configuration cote source

### nginx.conf
```nginx
http {
    log_format combined '$remote_addr - $remote_user [$time_local] '
                        '"$request" $status $body_bytes_sent '
                        '"$http_referer" "$http_user_agent"';
    access_log /var/log/nginx/access.log combined;
}
```

## Variables
| Variable         | Default                     | Description           |
|-----------------|-----------------------------|-----------------------|
| NGINX_ACCESS_LOG | /var/log/nginx/access.log  | Chemin du fichier log |

## Mapping OCSF
| nginx field    | OCSF                      |
|---------------|---------------------------|
| remote_addr   | src_endpoint.ip           |
| request       | http_request.http_method + url.path |
| status        | http_response.code        |
| body_bytes    | traffic.bytes_out         |
| user_agent    | http_request.user_agent   |

## Liens
- [nginx log module](https://nginx.org/en/docs/http/ngx_http_log_module.html)
- [Vector file source](https://vector.dev/docs/reference/configuration/sources/file/)
