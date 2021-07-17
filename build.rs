#[cfg(feature = "proto-compile")]
use std::fs;

fn main() {
    #[cfg(feature = "proto-compile")]
        {
            let out_path = "src/proto";
            let out_file_name = "src/proto/sentencepiece_model.proto";
            let proto_path = "sentencepiece/src/sentencepiece_model.proto";

            let metadata = fs::metadata(out_file_name);

            if metadata.is_err() {
                protobuf_codegen_pure::Codegen::new()
                    .out_dir(out_path)
                    .inputs(&[proto_path])
                    .include("sentencepiece/src")
                    .run()
                    .unwrap();
            }
        }
}