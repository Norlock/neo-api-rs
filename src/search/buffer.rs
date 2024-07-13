//use std::{pin::Pin, sync::Arc};

//use futures::{future::Shared, Future, FutureExt, TryFutureExt};

//struct TestComp {
    //text: String,
//}

//impl Test for TestComp {
    ////fn test(self: Box<&Self>) -> Pin<Box<dyn Future<Output = bool> + Send>> {
    ////fn test(self: Box<&Self>) -> impl Future<Output = bool> {
    //fn test(self: Box<Self>) -> Box<dyn Future<Output = Pin<Box<bool>>> + Send> {
        //Box::pin(async move {
            //tokio::fs::read("buffer.rs").await;

            //true
        //})
    //}
//}

//trait Test: Send {
    //fn test(self: Box<Self>) -> Box<dyn Future<Output = Pin<Box<bool>>> + Send>;
//}

//async fn test() {
    //let a: Box<dyn Test> = Box::new(TestComp {
        //text: "asd".to_string(),
    //});

    ////a.t\
    ////a.

    ////a.te

    //println!("{}", a.test().await);
//}
