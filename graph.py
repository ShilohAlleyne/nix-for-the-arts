# /// script
# requires-python = ">=3.12"
# dependencies = [
#   "pandas",
#   "plotly",
#   "Kaleido",
# ]
# ///


import pandas as pd
import plotly.express as px
import plotly.graph_objects as go
import plotly.io as pio

pio.renderers.default = "browser"

df = pd.read_csv("./data/pkgs.csv")

# Map SPDX license to parent family
def get_parent(spdx: str) -> str:
    # Take substring before first '-' if present, else whole string
    return spdx.split("-")[0] if "-" in spdx else spdx

df["parent"] = df["spdxId"].apply(get_parent)

# Count occurrences of each child license
license_counts = df["spdxId"].value_counts().reset_index()
license_counts.columns = ["spdxId", "count"]

# Merge counts back into df
df = df.merge(license_counts, on="spdxId", how="left")
df2 = df[["fullName", "parent", "count"]]
df2 = df2[["parent", "fullName", "count"]].rename(
    columns={
        "parent": "license",
        "fullName": "sub-license",
        "count": "count"
    }
)
df2 = df2.drop_duplicates()


fig = px.sunburst(
    df2,
    # Define the hierarchy: 'license' is the parent, 'sub-license' is the child
    path=['license', 'sub-license'],
    # Define the column that determines the size of each segment
    values='count',
    # Optional: Add a title to the chart
    title='The License and Sub-License Distribution of nixpkgs',
    color_discrete_sequence=px.colors.qualitative.Prism
)

fig.update_traces(
    textinfo="label+percent root",
    insidetextorientation="radial"   # make outer ring labels vertical
)

df2.to_csv("./data/pkgs_distribution.csv", index=False)
fig.show()

