SRC=docs.md
OUT=docs.html
OUT_DIR=website
PANDOC_OPTIONS=-t html5 --template=assets/template.html

%.html: %.md
	pandoc $(PANDOC_OPTIONS) -o $(OUT_DIR)/$@ $<

.PHONY: all clean watch

all: $(OUT)

# Reruns make on file change
watch:
	fswatch $(SRC) | xargs -I{} make

clean:
	rm $(OUT_DIR)/$(OUT)
