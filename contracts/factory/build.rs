extern crate protoc_rust;

fn main() -> Result<(), std::io::Error> {
    /*
        // this currently causes optimize to fail. re-enable if proto file changes.
        protoc_rust::Codegen::new()
            .out_dir("src/")
            .inputs(&["src/response.proto"])
            //  .include("protos")
            .run()?;
    */
    Ok(())
}
