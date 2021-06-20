import os
import re

NEURON_DIR = ""
MDBOOK_DIR = ""
titles = {}

for file in os.listdir(NEURON_DIR):
    if file == "index.md": continue
    if file.endswith(".md"):
        with open(os.path.join(NEURON_DIR, file)) as f:
            content = f.read()

        match = re.search(r"^---\n+^title: (.*?)(?:\n(.*))*?\n---", content, re.M)

        if match != None:
            titles[file] = match.group(1)
            content = content.replace(match.group(), "")
        else:
            titles[file] = file.replace(".md", "")

        with open(os.path.join(MDBOOK_DIR, "zettels", file), "w") as f:
            f.write(content)

with open(os.path.join(MDBOOK_DIR, "zettels", "SUMMARY.md"), "w") as f:
    f.write("# Summary\n\n")
    f.write("- [Welcome to my MDZK Zettelkasten](./index.md)")
    for file, title in titles.items():
        f.write(f"- [{title}](./{file})\n")
