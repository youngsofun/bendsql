const rust = import('./pkg');

rust
    .then(m => {
        let client = m.WasmClient.new();
        client.init("databend://root:@localhost:8000/default?sslmode=disable&wait_time_secs=10&login=disable");
        client.query("select 1").then((r) => {
            console.log(r);
            }
        )

        //console.log("The latest commit to the wasm-bindgen %s branch is:", data.name);
        //console.log("%s, authored by %s <%s>", data.commit.sha, data.commit.commit.author.name, data.commit.commit.author.email);
    })
    .catch(console.error);