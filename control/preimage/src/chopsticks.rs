use codec::Encode;
use handlebars::Handlebars;
use sp_crypto_hashing::blake2_256;
use std::collections::BTreeMap;
use std::io::prelude::*;
use std::{fs::File, path::PathBuf};

static TEMPLATE: &str = r#"
    const number = (await api.rpc.chain.getHeader()).number.toNumber()

    await api.rpc('dev_setStorage', {
        Preimage: {
            PreimageFor: [
                [
                [[{{PREIMAGE_HASH}}, {{PREIMAGE_SIZE}}]],
                {{PREIMAGE}}
                ]
            ],
            StatusFor: [
                [
                [{{PREIMAGE_HASH}}],
                {
                    Requested: {
                        count: 1,
                        len: {{PREIMAGE_SIZE}},
                    },
                },
                ],
            ],
        },
        Scheduler: {
            Agenda: [
                [
                [number + 1],
                [
                    {
                        call: {
                            Lookup: {
                                hash: {{PREIMAGE_HASH}},
                                len: {{PREIMAGE_SIZE}},
                            },
                        },
                        origin: {
                            system: 'Root',
                        },
                    },
                ],
                ],
            ],
        },
    })

    await api.rpc('dev_newBlock', { count: 1 })
"#;

pub fn make_chopsticks_script(preimage: &[u8], output_path: PathBuf) {
    let preimage_hash = format!("\"0x{}\"", hex::encode(blake2_256(preimage)));
    let preimage_size = format!("{}", preimage.len());
    let preimage_hex = format!("\"0x{}\"", hex::encode(preimage.to_owned().encode()));

    let mut substitutions = BTreeMap::new();
    substitutions.insert("PREIMAGE_HASH".to_string(), preimage_hash);
    substitutions.insert("PREIMAGE_SIZE".to_string(), preimage_size);
    substitutions.insert("PREIMAGE_HEX".to_string(), preimage_hex);

    let output = Handlebars::new()
        .render_template(TEMPLATE, &substitutions)
        .expect("render");
    let mut file = File::create(output_path).expect("create");
    file.write_all(output.as_bytes()).expect("write");
}
