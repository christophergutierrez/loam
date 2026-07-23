fn main() {
    // Thin CLI shell over loam_core. Subcommands land in later milestones.
    eprintln!("loam: use a subcommand (get | search | bundle | observe)");
    std::process::exit(2);
}
