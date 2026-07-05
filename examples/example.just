# This justfile has a speling mistake

alias b := buildd

greting := "Wolrd"

# Build the projct
buildd target="debugg":
    @echo "Bilding {{target}} now"
    # a shel comentt
    mkdir -p out

pythn-recipe:
    #!/usr/bin/env python3
    # python comentt here
    msg = "pythn strng"
    def my_functin():
        pass
