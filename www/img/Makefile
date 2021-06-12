

all: alice.svg.gz bob.svg.gz devil.svg.gz graduate.svg.gz guard.svg.gz duck.svg.gz judge.svg.gz santa.svg.gz sailor.svg.gz pilot.svg.gz police.svg.gz

%.svg.gz: %.svg
	zopfli $<

%.svg: %.pdf
	inkscape -z -f $< -l $@

%.pdf: %.tex
	pdflatex $<

%.tex:
	ln -s tikzpeople.tex $@

.PHONY: all
