SRC=docs.md
OUT=docs.html docs.1

OUTDIR_HTML=website
OUTDIR_TROFF=man

PANDOC_OPTIONS=--standalone
TROFF_OPTIONS=--to man
HTML_OPTIONS=--to html5\
			 --toc\
			 --toc-depth=2\
			 --section-divs\
			 --reference-location=section\
			 --lua-filter=assets/anchor-links.lua\
			 --template=assets/template.html

%.html: %.md
	@pandoc $(PANDOC_OPTIONS) $(HTML_OPTIONS) -o $(OUTDIR_HTML)/$@ $<\
	&& echo `date +%T`" - Built $@"

%.1: %.md
	@pandoc $(PANDOC_OPTIONS) $(TROFF_OPTIONS) -o $(OUTDIR_TROFF)/$@ $<\
	&& echo `date +%T`" - Built $@"

.PHONY: all clean man watch

all: $(OUT)

# Reruns make on file change
watch:
	@fswatch $(SRC) | xargs -I{} make -s

clean:
	@rm $(OUT_DIR)/$(OUT)\
	&& echo "Removed $(OUT_DIR)/$(OUT)"
