if [ $# -eq 2 ]; then
    crate_name="$1"
    semver="$2"
elif [ $# -eq 1 ]; then
    IFS="@"
    crate_name=$(echo "$1" | cut -d "@" -f 1)
    semver=$(echo "$1" | cut -d "@" -f 2)
else
    echo "Invalid argument count! Usage:"
    echo "    $0 <crate_name> <semver>"
    echo "or"
    echo "    $0 <crate_name>@<semver>"
    exit 1
fi

tar_file="$crate_name"-"$semver".gz

echo Getting source code of "$crate_name" v"$semver" ...
if [ ! -f "$tar_file" ]; then
    cargo download "$crate_name"=="$semver" > "$tar_file"
fi

echo Extracting source code of "$crate_name"-"$semver" ...
tar -zxf "$tar_file" -C modules/
mv modules/"$crate_name-$semver" modules/ak_addition/ak_"$crate_name"

echo Cleaning up ...
rm "$tar_file"