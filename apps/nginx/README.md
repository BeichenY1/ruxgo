# Build and Run [`nginx`](https://github.com/nginx/nginx) on RuxOS

Firstly, you need to copy `config_linux.toml` from `ruxgo/apps/nginx/ruxos` and place it in the `ruxos/apps/c/rux-nginx` at the same level as `nginx-1.24.0`.

Then, switch to `ruxos/apps/c/rux-nginx` directory. If `nginx-1.24.0` does not exist, execute the following prerequisite commands (if it does, it is not required):

```bash
wget https://nginx.org/download/nginx-1.24.0.tar.gz
tar -zxvf nginx-1.24.0.tar.gz && rm -f nginx-1.24.0.tar.gz
```

After that, you need to execute the following command:

```bash
./create_nginx_img.sh
```

Finally, execute the following commands to build and run it:

```bash
# Build and Run
ruxgo -b
ruxgo -r
```
