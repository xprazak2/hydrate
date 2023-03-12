use hydrate::cache::Cache;


#[tokio::main]
async fn main() -> anyhow::Result<()>{
    let mut cache = Cache::new();

    let mut foo_num = 0;
    let foo = move || -> String {
        let mut vc = vec![];
        vc.push("f");
        for _ in 0..foo_num {
            vc.push("o");
        }
        foo_num += 1;
        vc.join("")
    };

    let mut bar_num = 1;
    let bar = move || -> String {
        let mut vc = vec![];
        for _ in 0..bar_num {
            vc.push("bar");
        }
        bar_num += 1;
        vc.join("")
    };

    cache.register(foo, "foo", 20, 3).await?;
    cache.register(bar, "bar", 20, 5).await?;

    loop {}
}
