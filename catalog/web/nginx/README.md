# Nginx — combined access log

## Description
Collects nginx access logs (combined format).
Normalized to OCSF class 4002 (HTTP Activity).

## Expected format
```
10.0.0.1 - user [14/Apr/2024:10:23:45 +0000] "GET /path HTTP/1.1" 200 1234 "referer" "user-agent"
```

## Source-side configuration

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
| NGINX_ACCESS_LOG | /var/log/nginx/access.log  | Log file path         |

## OCSF mapping
| nginx field    | OCSF                      |
|---------------|---------------------------|
| remote_addr   | src_endpoint.ip           |
| request       | http_request.http_method + url.path |
| status        | http_response.code        |
| body_bytes    | traffic.bytes_out         |
| user_agent    | http_request.user_agent   |

## Links
- [nginx log module](https://nginx.org/en/docs/http/ngx_http_log_module.html)
- [Vector file source](https://vector.dev/docs/reference/configuration/sources/file/)
