"""
Module for downloading and processing model quality metrics based on human preference using the
LMSYS Chatbot Arena Leaderboard https://chat.lmsys.org/?leaderboard.

@misc{chiang2024chatbot,
    title={Chatbot Arena: An Open Platform for Evaluating LLMs by Human Preference},
    author={Wei-Lin Chiang and Lianmin Zheng and Ying Sheng and Anastasios Nikolas Angelopoulos and Tianle Li and Dacheng Li and Hao Zhang and Banghua Zhu and Michael Jordan and Joseph E. Gonzalez and Ion Stoica},
    year={2024},
    eprint={2403.04132},
    archivePrefix={arXiv},
    primaryClass={cs.AI}
}
"""

from datetime import datetime, timedelta
from glob import glob
from os import path, makedirs
import pickle

import pandas as pd
import requests

# Local directory with downloaded files
downloads = path.join(path.dirname(path.abspath(__file__)), "downloads", "lmsys")
makedirs(downloads, exist_ok=True)


def download_date(date: str):
    """
    Download files for a date  (if not already present)
    """
    for file_name in [f"elo_results_{date}.pkl", f"leaderboard_table_{date}.csv"]:
        # Account for differences in filename for this date
        if file_name == "leaderboard_table_20240403.csv":
            file_name = "leaderboard_table_20240404.csv"

        file_path = path.join(downloads, file_name)
        if path.exists(file_path):
            continue

        url = f"https://huggingface.co/spaces/lmsys/chatbot-arena-leaderboard/resolve/main/{file_name}"
        response = requests.get(url, stream=True)

        if response.status_code == 404:
            continue

        response.raise_for_status()

        print(f"Downloading {file_name}")
        with open(file_path, "wb") as file:
            for chunk in response.iter_content(chunk_size=8192):
                if chunk:
                    file.write(chunk)


def download_all():
    """
    Download data for all dates
    """

    # Dates at https://huggingface.co/spaces/lmsys/chatbot-arena-leaderboard/tree/main
    dates = [
        "20230619",
        "20230717",
        "20230802",
        "20230905",
        "20231002",
        "20231108",
        "20231116",
        "20231206",
        "20231215",
        "20231220",
        "20240109",
        "20240118",
        "20240125",
        "20240202",
        "20240215",
        "20240305",
        "20240307",
        "20240313",
        "20240326",
        "20240403",
        "20240410",
        "20240411",
        "20240413",
        "20240418",
        "20240419",
        "20240422",
        "20240426",
        "20240501",
        "20240508",
        "20240515",
        "20240516",
        "20240519",
        "20240520",
        "20240527",
        "20240602",
        "20240606",
        "20240611",
        "20240617",
        "20240621",
        "20240623",
    ]
    [download_date(date) for date in dates]

    # Attempt to get any other dates not included in above list
    date = datetime(2024, 6, 24)
    while date <= datetime.now():
        download_date(date.strftime("%Y%m%d"))
        date += timedelta(days=1)


def extract_pickles():
    """
    Extract leaderboard dataframes from the downloaded pickle files

    Both pandas and plotly are required to load the pickle files
    """

    fulls = []
    codings = []
    for file_name in sorted(glob(path.join(downloads, "elo_results_*.pkl"))):

        with open(file_name, "rb") as file:
            data = pickle.load(file)

        if "full" in data:
            if "leaderboard_table_df" in data["full"]:
                full = data["full"]["leaderboard_table_df"]
            else:
                raise KeyError(f"Keys of data[full]: {data['full'].keys()}")
        elif "leaderboard_table_df" in data:
            full = data["leaderboard_table_df"]
        else:
            continue
        
        full['date'] = file_name
        fulls.append(full)

        if "coding" in data:
            if "leaderboard_table_df" in data["coding"]:
                coding = data["coding"]["leaderboard_table_df"]
                coding['date'] = file_name
                codings.append(coding)

    fulls = pd.concat(fulls)
    fulls.to_csv("overall.csv")

    codings = pd.concat(codings)
    codings.to_csv("coding.csv")

if __name__ == "__main__":
    download_all()
    extract_pickles()
