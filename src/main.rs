fn main() -> anyhow::Result<()> {
    let code = skillhub::run_with_args(std::env::args_os())?;
    std::process::exit(code);
}
