#!/usr/bin/env bash
set -eu

source scripts/generate-beefy-fixture.sh

from_benchmark=true
gas_report="/tmp/beefy_gas_report"

for mininum_signature in {17..30}
do
    echo -e "\n********************** begin test *****************************\n"
    echo "minimum require signature: $mininum_signature"
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



