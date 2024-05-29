
use tracing::{debug, span, Instrument, Level};

use crate::make_span;



#[cfg(test)]
mod global;

#[tokio::test]
async fn test_span() {
    
    tracing_subscriber::fmt()
    .with_max_level(tracing::metadata::LevelFilter::DEBUG)
    .with_target(false)
    .init();

    say_hello().await;

    {
        const ROOT: &'static str = "root";
        // "root" 必须是常量
        let span = span!(tracing::Level::DEBUG, ROOT);
        global::init_span(span);
    }

    debug!("============ span1 =================");
    say_hello()
    .instrument(make_span!())
    .await;
    
    debug!("============ span2 =================");
    say_hello()
    .instrument(make_span!("mod", session="ss001"))
    .await;

    debug!("============ span3 =================");
    let task = tokio::spawn(async move {
        say_hello().await
    }.instrument(make_span!("mod", session="ss001")));

    task.await.unwrap();

    {
        let span = global::get_span();

        let span1 = span!(parent: span.clone(), Level::DEBUG, "", "mc001");
        say_hello().instrument(span1.clone()).await;
    
        let span2 = span!(parent: span1.clone(), Level::DEBUG, "", "mc002");
        say_hello().instrument(span2.clone()).await;
    }


}

async fn say_hello() {
    debug!("Hello");
    say_how_are_you().await;
}

async fn say_how_are_you() {
    debug!("How are you");
}