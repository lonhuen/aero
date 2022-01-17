#!/bin/sh
#if [[ $# != '0' ]]
#then
#        echo "usage: $0"
#        exit 1
#fi

GRAPHNAME=`basename $0 .sh`
EXPNAME="${GRAPHNAME%.*}"
EPS=graphs/$GRAPHNAME.eps
touch plot.plt

echo "
set terminal postscript eps 'Times-Roman,30' color 
set terminal postscript eps 'Times-Roman,30' color size 6.3,4
set output 'graphs/$GRAPHNAME.eps'
set xrange [0.00001 : 0.000000000001] reverse
set xtics nomirror font 'Times-Roman, 45' offset 0,-0.1
set nomxtics
set logscale x 100
set xlabel \"Failure Probability\" font 'Times-Roman,45'
#set yrange [ 0.01 : 20637 ]
set ylabel \"Committee Network (MB)\\\n \" font 'Times-Roman,39' offset -0.0,0.0
set ytics nomirror font 'Times-Roman, 45'
set nomytics
set ytics 10
#set format y '2^{%T}'
#set format y '%E'
set format x '10^{%T}'
set logscale y 10
set grid noxtics noytics
set grid ytics lw -1
set border 3 lw 4
set key at 0.00000000005,1000
#set key top left
set bmargin 3.5
set pointsize 2.0
set style function linespoints
set style line 1 lc rgb 'blue' dt 1 lt 1 lw 9 pt 12 pi -1 ps 3.5
set style line 2 lc rgb 'red' dt 1 lt 1 lw 7 pt 9 pi -1 ps 3.5
set style line 3 lc rgb 'black' dt 1 lt 1 lw 7 pt 4 pi -1 ps 3.5
set style line 4 lc rgb 'orange' dt 1 lt 1 lw 9 pt 7 pi -1 ps 3.5
plot \
'dat/${EXPNAME}.dat' using 1:(\$2/1000000) with linespoints ls 1 title 'f=0.05, # of committees=  1',\
'dat/${EXPNAME}.dat' using 1:(\$3/1000000) with linespoints ls 2 title 'f=0.05, # of committees=10',\
'dat/${EXPNAME}.dat' using 1:(\$4/1000000) with linespoints ls 3 title 'f=0.03, # of committees=  1',\
'dat/${EXPNAME}.dat' using 1:(\$5/1000000) with linespoints ls 4 title 'f=0.03, # of committees=10',\
" > plot.plt
gnuplot plot.plt
epspdf $EPS
rm $EPS
