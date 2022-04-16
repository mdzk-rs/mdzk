use crate::error::Result;
use anyhow::Context;
use ignore::{types::TypesBuilder, WalkBuilder};
use std::path::Path;
use std::sync::mpsc;

const COPY_GLOBS: [&str; 16] = [
    "*.apng", "*.avif", "*.bmp", "*.cur", "*.gif", "*.ico", "*.jfif", "*.jpg", "*.jpeg", "*.pjp",
    "*.pjpeg", "*.png", "*.svg", "*.tif", "*.tiff", "*.webp",
];

pub(crate) fn copy_other_files<F, T>(from: F, to: T) -> Result<()>
where
    F: AsRef<Path>,
    T: AsRef<Path>,
{
    let from = from.as_ref();
    let to = to.as_ref();

    if from == to {
        return Ok(());
    }

    let walker = {
        let mut builder = TypesBuilder::new();
        for glob in COPY_GLOBS {
            builder.add("copy", glob)?;
        }

        let types = builder.select("copy").build()?;

        WalkBuilder::new(from)
            .hidden(true)
            .types(types)
            .build_parallel()
    };

    let (sender, reciever) = mpsc::channel();

    walker.run(|| {
        let sender = sender.clone();

        Box::new(move |e| {
            let copy_file = || -> Result<()> {
                if let Ok(e) = e {
                    let path = e.path();
                    if !path.is_dir() {
                        let out_path = to.join(path.strip_prefix(from)?);
                        if let Some(parent) = out_path.parent() {
                            if !parent.exists() {
                                std::fs::create_dir_all(parent)?;
                            }
                        }

                        std::fs::copy(path, &out_path).with_context(|| {
                            format!("Error when copying {path:?} to {out_path:?}")
                        })?;
                    }
                }
                Ok(())
            };

            match copy_file() {
                Ok(()) => ignore::WalkState::Continue,
                Err(err) => {
                    sender.send(err).unwrap();
                    ignore::WalkState::Quit
                }
            }
        })
    });

    drop(sender);

    if let Some(err) = reciever.into_iter().next() {
        return Err(err);
    }

    Ok(())
}
