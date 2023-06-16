#!/bin/env bash
set -x
(
echo -e 'id\tname\turl' ; (
curl https://en.wikipedia.org/wiki/List_of_musicals:_A_to_L ; 
curl https://en.wikipedia.org/wiki/List_of_musicals:_M_to_Z 
) \
   | grep '<td><i><a href' \
   | sed -E 's/.*title="([^"]*)"[^>]*>([^<]*).*/\2\t\1/' \
   | nl -w1 "-s$(echo -e '\t')"
) \
   | tee musicals.csv
