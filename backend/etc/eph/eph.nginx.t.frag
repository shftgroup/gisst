  location = /$ROOT/$BRANCH {
    return 301 /$ROOT/$BRANCH/;
  }
  location /$ROOT/$BRANCH/ {
        proxy_redirect off;
        proxy_set_header   X-Forwarded-For $remote_addr;
        proxy_set_header   Host $host;
        proxy_pass_header  X-Accepted-Encoding;
        proxy_pass http://localhost:$PORT/;
        proxy_pass_request_headers on;
        client_max_body_size  32M;
        proxy_ssl_verify off;
        proxy_ssl_session_reuse on;
        rewrite  ^/$ROOT\/$BRANCH/(.*)  /$1 break;
  }
