import os
import re
from datetime import datetime
import io
import click
import ftplib
from dateutil.parser import parse as parse_datetime
from typing import List, Optional


class FTPHandler:
    def __init__(self, host: str, username: str, password: str):
        self.host = host
        self.username = username
        self.password = password

    def __enter__(self) -> "FTPHandler":
        (address, port) = self.host.split(":")
        self.ftp = ftplib.FTP()
        self.ftp.connect(host=address, port=int(port))
        self.ftp.login(user=self.username, passwd=self.password)
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        self.ftp.quit()

    def list_files(
        self,
        path: str,
        regex_pattern: str,
        start_time: Optional[datetime] = None,
        end_time: Optional[datetime] = None,
    ) -> List[str]:
        files = self.ftp.nlst(path)

        filtered_files = []
        for file in files:
            if re.search(regex_pattern, file):
                if start_time or end_time:
                    modified_time = self.get_modified_time(file)
                    if start_time and modified_time < start_time:
                        continue
                    if end_time and modified_time > end_time:
                        continue
                filtered_files.append(file)
        return filtered_files

    def get_modified_time(self, filename: str) -> datetime:
        modified_time_str = self.ftp.sendcmd(f"MDTM {filename}")[4:]
        modified_time = datetime.strptime(modified_time_str, "%Y%m%d%H%M%S")
        return modified_time

    def transfer_files(
        self, files: List[str], remote_path: str, remote_ftp_handler: "FTPHandler"
    ):
        buffer = io.BytesIO()
        remote_ftp_handler.ftp.cwd(remote_path)
        for file in files:
            try:
                base_name = os.path.basename(file)
                self.ftp.retrbinary(f"RETR {file}", buffer.write)
                buffer.seek(0)
                remote_ftp_handler.ftp.storbinary(f"STOR {base_name}", buffer)
            except Exception as e:
                print(f"{str(e)}")
            finally:
                buffer.flush()

    def close(self):
        self.ftp.quit()


@click.command()
@click.option("--source-host", required=True, type=str)
@click.option("--source-user", required=True, type=str)
@click.option("--source-pass", required=True, type=str)
@click.option("--target-host", required=True, type=str)
@click.option("--target-user", required=True, type=str)
@click.option("--target-pass", required=True, type=str)
@click.option("--source-path", required=True, type=str)
@click.option("--target-path", required=True, type=str)
@click.option("--regex-pattern", required=True, type=str)
@click.option("--start-time", default=None, type=str)
@click.option("--end-time", default=None, type=str)
def main(
    source_host: str,
    source_user: str,
    source_pass: str,
    target_host: str,
    target_user: str,
    target_pass: str,
    source_path: str,
    target_path: str,
    regex_pattern: str,
    start_time: Optional[str],
    end_time: Optional[str],
):
    with FTPHandler(source_host, source_user, source_pass) as source_ftp:
        with FTPHandler(target_host, target_user, target_pass) as destination_ftp:
            start_path = source_path
            regex = regex_pattern
            start_time = (
                parse_datetime(start_time)
                if start_time
                else parse_datetime("2024-01-01T00:00:00")
            )
            end_time = (
                parse_datetime(end_time)
                if end_time
                else parse_datetime("2025-01-01T00:00:00")
            )

            filtered_files = source_ftp.list_files(
                start_path, regex, start_time, end_time
            )

            source_ftp.transfer_files(filtered_files, target_path, destination_ftp)


if __name__ == "__main__":
    main()
