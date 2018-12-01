#!/bin/sh

code=0

for file in *.rs
do
    printf "Testing %-30s…" "$file"

    output=$(rustfmt --check --color=always --edition=2018 "$file")
    if [ "$?" != "0" ]
    then
        echo "fail, style error:\n$output"
        code=1
        continue
    fi

    tmp="$(mktemp -d)"
    output="$(rustc --color=always --edition=2018 -o "$tmp/main" $file)"
    if [ "$?" != "0" ]
    then
        echo "fail, rustc error:\n$output"
        code=1
        continue
    fi


    output=$("$tmp/main")
    exit_code="$?"
    if [ "$exit_code" != "0" ]
    then
        echo "fail, exit code $exit_code:\n$output"
        code=1
        continue
    fi

    expected="Encrypted: lahhk"
    if [ "$output" != "$expected" ]
    then
        echo "fail, output:\nexpected '$expected' but got '$output'"
        code=1
        continue
    fi

    echo "ok"
done

exit $code
