WATCH=docs.md man.md
OUT=website/docs.html website/man.html

PANDOC_OPTIONS=--standalone
HTML_OPTIONS=--to html5\
			 --toc\
			 --toc-depth=2\
			 --section-divs\
			 --reference-location=section\
			 --lua-filter=assets/anchor-links.lua\
			 --template=assets/template.html

.PHONY: all clean watch

all: $(OUT)

website/docs.html: docs.md
	@pandoc $(PANDOC_OPTIONS) $(HTML_OPTIONS) -o $@ $<\
	&& echo `date +%T`" - Built $@"

website/man.html: man.md
	@pandoc $(PANDOC_OPTIONS) $(HTML_OPTIONS) -o $@ $<\
	&& echo `date +%T`" - Built $@"

# Reruns make on file change
watch:
	@fswatch $(WATCH) | xargs -I{} make -s

clean:
	@rm $(OUT)\
	&& echo "Removed $(OUT)"
