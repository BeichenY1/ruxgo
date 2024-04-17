# Load your app executable to run on RuxOS

Inside this is a configuration template for loading your app executable, you just need to fill in your app path in the app field in [build].

To load and run your app executable, you first need to create your app directory under **`ruxos/apps/c/`**, then put your custom toml files into it, and then build and run with the following commands:

```bash
# Build and Run
ruxgo -b
ruxgo -r
```

**Note**: 

The app executable needs to be compiled by musl.