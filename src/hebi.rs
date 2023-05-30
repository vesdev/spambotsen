use std::{sync::mpsc, time::Duration};

use hebi::Hebi;

pub async fn eval_hebi(source: String) -> String {
    let mut hebi = Hebi::builder().output(Vec::<u8>::new()).finish();

    let (sender, receiver) = mpsc::channel();
    let t = std::thread::spawn(move || {
        sender.send((
            match hebi.eval(&source) {
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

    let (result, output) = match receiver.recv_timeout(Duration::from_secs(10)) {
        Ok(result) => result,
        Err(_) => {
            drop(receiver);
            drop(t);

            ("Request timed out".to_string(), "None".to_string())
        }
    };

    if output.is_empty() {
        format!("## Result:\n{result}")
    } else {
        format!("## Result:\n{result}\n## Output:\n```{output}```")
    }
}
