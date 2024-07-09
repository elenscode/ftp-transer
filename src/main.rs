use chrono::prelude::*;
use clap::Parser;
use regex::Regex;
use suppaftp::FtpStream;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    source_host: String,

    #[arg(long)]
    source_user: String,

    #[arg(long)]
    source_pass: String,

    #[arg(long)]
    target_host: String,

    #[arg(long)]
    target_user: String,

    #[arg(long)]
    target_pass: String,

    #[arg(long)]
    source_path: String,

    #[arg(long)]
    target_path: String,

    #[arg(long)]
    regex_pattern: String,

    #[arg(long)]
    start_time: Option<String>,

    #[arg(long)]
    end_time: Option<String>,
}

trait FtpFilter {
    fn filter_files(&mut self, source_path: &str, regex_pattern: &str) -> Vec<String>;
    fn filter_by_time(
        &mut self,
        files: Vec<String>,
        start_time: NaiveDateTime,
        end_time: NaiveDateTime,
    ) -> Vec<String>;
}

struct FtpClient {
    ftp_stream: FtpStream,
}

impl FtpClient {
    fn new(host: &str, user: &str, pass: &str) -> Self {
        let mut ftp_stream = FtpStream::connect(host).unwrap();
        ftp_stream.login(user, pass).unwrap();
        FtpClient { ftp_stream }
    }

    fn upload_to_target(
        &mut self,
        target_ftp: &mut FtpStream,
        files: Vec<String>,
        target_path: &str,
    ) {
        for file in files {
            let mut cursor = self.ftp_stream.retr_as_buffer(&file).unwrap();

            let base_name = file.split('/').last().unwrap();
            print!("{}", base_name);
            target_ftp.cwd(target_path).unwrap();
            target_ftp.put_file(base_name, &mut cursor).unwrap();
        }
    }
}

impl FtpFilter for FtpClient {
    fn filter_files(&mut self, source_path: &str, regex_pattern: &str) -> Vec<String> {
        let files = self.ftp_stream.nlst(Some(source_path)).unwrap();
        let regex = Regex::new(regex_pattern).unwrap();
        files
            .into_iter()
            // .filter_map(|file| {
            //     file.split_whitespace()
            //         .last()
            //         .map(|filename| filename.to_string())
            // })
            .filter(|filename| regex.is_match(filename))
            .collect()
    }

    fn filter_by_time(
        &mut self,
        files: Vec<String>,
        start_time: NaiveDateTime,
        end_time: NaiveDateTime,
    ) -> Vec<String> {
        files
            .into_iter()
            .filter(|file| {
                if let Ok(modified_time) = self.ftp_stream.mdtm(file) {
                    return modified_time >= start_time && modified_time <= end_time;
                }
                false
            })
            .collect()
    }
}

fn main() {
    let args = Args::parse();
    "/target/path";
    let source_host = &args.source_host;
    let source_user = &args.source_user;
    let source_pass = &args.source_pass;
    let target_host = &args.target_host;
    let target_user = &args.target_user;
    let target_pass = &args.target_pass;
    let source_path = &args.source_path;
    let target_path = &args.target_path;
    let regex_pattern = &args.regex_pattern;

    let start_time = match args.start_time.as_deref() {
        Some(v) => NaiveDateTime::parse_from_str(v, "%Y-%m-%dT%H:%M:%S").unwrap(),
        None => NaiveDateTime::parse_from_str("2024-01-01T00:00:00", "%Y-%m-%dT%H:%M:%S").unwrap(),
    };
    let end_time = match args.start_time.as_deref() {
        Some(v) => NaiveDateTime::parse_from_str(v, "%Y-%m-%dT%H:%M:%S").unwrap(),
        None => NaiveDateTime::parse_from_str("2024-12-31T23:59:00", "%Y-%m-%dT%H:%M:%S").unwrap(),
    };
    let mut source_ftp = FtpClient::new(source_host, source_user, source_pass);
    let mut target_ftp = FtpClient::new(target_host, target_user, target_pass).ftp_stream;

    let filtered_files = source_ftp.filter_files(source_path, regex_pattern);
    println!("{:?}", filtered_files);
    let time_filtered_files = source_ftp.filter_by_time(filtered_files, start_time, end_time);
    println!("{:?}", time_filtered_files);

    source_ftp.upload_to_target(&mut target_ftp, time_filtered_files, target_path);
    let _ = source_ftp.ftp_stream.quit();
    let _ = target_ftp.quit();
}
