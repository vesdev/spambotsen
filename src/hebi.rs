use std::fmt::Display;

use hebi::Hebi;

struct EvalResult {
    result: String,
    output: String,
    disassembly: Option<String>,
}

impl Display for EvalResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(disassembly) = &self.disassembly {
            write!(f, "## Disassembly:\n```{}```\n", disassembly)?;
        }
        if self.output.is_empty() {
            write!(f, "## Result:\n```{}```", self.result)?;
        } else {
            write!(
                f,
                "## Result:\n```{}```\n## Output:\n```{}```",
                self.result, self.output
            )?;
        }

        Ok(())
    }
}

pub async fn eval_hebi(source: String, disassemble: bool) -> String {
    let (tx, mut rx) = tokio::sync::oneshot::channel();
    let mut hebi = Hebi::builder().output(Vec::<u8>::new()).finish();

    tokio::spawn(async move {
        tx.send(EvalResult {
            disassembly: if disassemble {
                Some(hebi.compile(&source).unwrap().disassemble().to_string())
            } else {
                None
            },
            result: match hebi.eval_async(&source).await {
                Ok(value) => format!("Value: {value:#?}"),
                Err(e) => e.report(&source, false),
            },
            output: String::from_utf8(
                hebi.global()
                    .output()
                    .as_any()
                    .downcast_ref::<Vec<u8>>()
                    .cloned()
                    .unwrap(),
            )
            .unwrap(),
        })
    });

    let sleep = tokio::time::sleep(tokio::time::Duration::from_secs(10));
    tokio::pin!(sleep);

    tokio::select! {
        Ok(v) = &mut rx => {
            v.to_string()
        },
        _ = &mut sleep => {
            "Request timed out".to_string()
        }
    }
}
