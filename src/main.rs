//! BeniUnityEmu CLI – Unity Emulation Runtime by Beni

use std::env;
use std::io::{self, Write};
use std::process;

use beni_unity_emu::{
    ensure_directories, enrich_app_info, extract_app, find_app, inspect_app, run_app, scan_apps,
    AppInfo,
};

#[derive(Debug)]
enum Command {
    Scan,
    Inspect(String),
    Extract(String),
    Run(String),
    Interactive,
}

fn parse_args() -> Command {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Command::Interactive;
    }
    match args[1].as_str() {
        "--scan" | "scan" => Command::Scan,
        "--inspect" | "inspect" => {
            if args.len() < 3 {
                eprintln!("[ERROR] --inspect requires an application name");
                process::exit(1);
            }
            Command::Inspect(args[2].clone())
        }
        "--extract" | "extract" => {
            if args.len() < 3 {
                eprintln!("[ERROR] --extract requires an application name");
                process::exit(1);
            }
            Command::Extract(args[2].clone())
        }
        "--run" | "run" => {
            if args.len() < 3 {
                eprintln!("[ERROR] --run requires an application name");
                process::exit(1);
            }
            Command::Run(args[2].clone())
        }
        "--help" | "-h" | "help" => {
            println!("BeniUnityEmu - Unity Emulation Runtime by Beni");
            println!();
            println!("Usage:");
            println!("  BeniUnityEmu                  Interactive menu");
            println!("  BeniUnityEmu --scan           List applications");
            println!("  BeniUnityEmu --inspect <app>  Inspect APK/IPA");
            println!("  BeniUnityEmu --extract <app>  Extract to cache");
            println!("  BeniUnityEmu --run <app>      Attempt to run");
            println!();
            println!("GUI launcher: BeniAppPicker.exe");
            process::exit(0);
        }
        other => {
            eprintln!("[ERROR] Unknown argument: {}", other);
            eprintln!("Use --help for usage.");
            process::exit(1);
        }
    }
}

fn main() {
    let cmd = parse_args();

    println!("========================================");
    println!("  BeniUnityEmu");
    println!("  Unity Emulation Runtime");
    println!("  Made by Beni");
    println!("========================================");
    println!();

    ensure_directories();

    match cmd {
        Command::Scan => scan_and_list(),
        Command::Inspect(name) => {
            if let Some(mut app) = find_app(&name) {
                enrich_app_info(&mut app);
                print!("{}", inspect_app(&app));
            } else {
                eprintln!("[ERROR] Application not found: {}", name);
                process::exit(1);
            }
        }
        Command::Extract(name) => {
            if let Some(app) = find_app(&name) {
                match extract_app(&app) {
                    Ok(msg) => println!("{}", msg),
                    Err(e) => {
                        eprintln!("[ERROR] {}", e);
                        process::exit(1);
                    }
                }
            } else {
                eprintln!("[ERROR] Application not found: {}", name);
                process::exit(1);
            }
        }
        Command::Run(name) => {
            if let Some(mut app) = find_app(&name) {
                enrich_app_info(&mut app);
                match run_app(&app) {
                    Ok(msg) => println!("[GAME] {}", msg),
                    Err(e) => {
                        eprintln!("[ERROR] {}", e);
                        process::exit(1);
                    }
                }
            } else {
                eprintln!("[ERROR] Application not found: {}", name);
                process::exit(1);
            }
        }
        Command::Interactive => interactive_menu(),
    }
}

fn scan_and_list() {
    println!("[INFO] Searching for applications...");
    let apps = scan_apps();
    if apps.is_empty() {
        println!("[INFO] No applications found.");
        println!("[INFO] Place APKs/IPAs in ./BeniApps/ (or BeniAPK_Apps / BeniIOS_Apps)");
        return;
    }
    println!();
    for (i, app) in apps.iter().enumerate() {
        println!("[{}] {} ({})", i + 1, app.name, app.format_label());
    }
    println!();
}

fn interactive_menu() {
    loop {
        println!("Searching for applications...");
        let apps = scan_apps();

        if apps.is_empty() {
            println!("[INFO] No applications found.");
            println!("[INFO] Place APKs/IPAs in ./BeniApps/");
            println!();
            println!("Press Enter to exit...");
            let mut buf = String::new();
            let _ = io::stdin().read_line(&mut buf);
            break;
        }

        println!();
        for (i, app) in apps.iter().enumerate() {
            println!("[{}] {} ({})", i + 1, app.name, app.format_label());
        }
        println!("[0] Exit");
        print!("Select application: ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }
        let choice = input.trim();

        if choice == "0" || choice.eq_ignore_ascii_case("exit") || choice.eq_ignore_ascii_case("q")
        {
            break;
        }

        if let Ok(idx) = choice.parse::<usize>() {
            if idx >= 1 && idx <= apps.len() {
                app_submenu(&apps[idx - 1]);
            } else {
                println!("[ERROR] Invalid selection.");
            }
        } else {
            println!("[ERROR] Invalid input.");
        }
        println!();
    }
}

fn app_submenu(app: &AppInfo) {
    let mut info = app.clone();
    enrich_app_info(&mut info);

    loop {
        println!();
        println!("========================================");
        println!("  Application Information");
        println!("========================================");
        println!("Name:         {}", info.name);
        println!(
            "Package:      {}",
            info.package.as_deref().unwrap_or("(unknown)")
        );
        println!(
            "Version:      {}",
            info.version.as_deref().unwrap_or("(unknown)")
        );
        println!(
            "Architecture: {}",
            info.architecture.as_deref().unwrap_or("(unknown)")
        );
        println!("Format:       {}", info.format_label());
        println!("Size:         {}", info.size_label());
        println!("Status:       {}", info.status_label());
        println!("Native libraries:");
        if info.native_libs.is_empty() {
            println!("  (none detected yet)");
        } else {
            for lib in &info.native_libs {
                println!("  - {}", lib);
            }
        }
        println!();
        println!("[1] Inspect");
        println!("[2] Extract to cache");
        println!("[3] Run");
        println!("[4] Back");
        print!("Select option: ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }
        match input.trim() {
            "1" => print!("{}", inspect_app(&info)),
            "2" => match extract_app(&info) {
                Ok(m) => println!("{}", m),
                Err(e) => eprintln!("[ERROR] {}", e),
            },
            "3" => match run_app(&info) {
                Ok(m) => println!("[GAME] {}", m),
                Err(e) => eprintln!("[ERROR] {}", e),
            },
            "4" | "b" | "B" => break,
            _ => println!("[ERROR] Invalid selection."),
        }
    }
}
