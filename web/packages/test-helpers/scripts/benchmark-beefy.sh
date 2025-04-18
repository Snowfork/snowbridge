#!/usr/bin/env bash
set -eu

from_benchmark=true

source scripts/generate-beefy-fixture.sh

gas_report="/tmp/beefy_gas_report"

signatures=$(echo 17-30,50,60,70,80,90,100 | perl -pe 's/(\d+)-(\d+)/join(",",$1..$2)/eg')
echo $signatures

for mininum_signature in $(echo $signatures | sed "s/,/ /g")
do
    echo -e "\n********************** begin test *****************************\n"
    echo "minimum require signature: $mininum_signature, waiting for benchmark..."
    generate_beefy_fixture $mininum_signature > $gas_report 2>&1
    generate_beefy_gas_report $mininum_signature >> $gas_report 2>&1
    gas_cost=$(cat $gas_report | grep "| submitFinal" | awk -F "|" '{print $6}' | sed 's/ //g')
    computed_signature=$(cat $gas_report | grep "computed required signatures:" | awk -F ":" '{print $2}' | sed 's/ //g')
    if [ $mininum_signature == 17 ]; then
        xarray+="$computed_signature"
        yarray+="$gas_cost"
    else
        xarray+=",$computed_signature"
        yarray+=",$gas_cost"
    fi
    array+=( "minimum signature: $mininum_signature, computed signature: $computed_signature, gas cost: $gas_cost" )
    echo -e "\n********************** end test *****************************\n"
done

echo -e "\n*********** xaxis *********"
printf '%s\n' "${xarray[@]}"

echo -e "\n********** y axis *********"
printf '%s\n' "${yarray[@]}"

echo -e "\n********** cost of submitFinal by signatures *********"
printf '%s\n' "${array[@]}"



