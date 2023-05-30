use std::{sync::mpsc, time::Duration};

use hebi::Hebi;

pub async fn eval_hebi(source: String) -> String {
    let (tx, mut rx) = tokio::sync::oneshot::channel();
    let mut hebi = Hebi::builder().output(Vec::<u8>::new()).finish();
    tokio::spawn(async move {
        tx.send((
            match hebi.eval_async(&source).await {
                Ok(value) => format!("Value: {value:#?}"),
                Err(e) => e.report(&source, false),
            },
            String::from_utf8(
                hebi.global()
                    .output()
                    .as_any()
                    .downcast_ref::<Vec<u8>>()
                    .cloned()
                    .unwrap(),
            )
            .unwrap(),
        ))
    });
    let sleep = tokio::time::sleep(Duration::from_secs(10));
    tokio::pin!(sleep);

    let (output, result) = tokio::select! {
        Ok(v) = &mut rx => {
            v
        },
        _ = &mut sleep => {
            ("Request timed out".to_string(),
            "None".to_string())
        }
    };

    if output.is_empty() {
        format!("## Result:\n{result}")
    } else {
        format!("## Result:\n{result}\n## Output:\n```{output}```")
    }
}
