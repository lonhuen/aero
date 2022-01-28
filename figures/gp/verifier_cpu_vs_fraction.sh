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
set xrange [ 0.000001: 1 ]
set xtics nomirror font 'Times-Roman, 35' offset 0,-0.1
set nomxtics
set logscale x 100
set xlabel \"Device sampling probability (q)\" font 'Times-Roman,35'
#set yrange [ 0 : 1600]
set ylabel \"Verifier CPU (sec.)\\\n \" font 'Times-Roman,35' offset -1,0.0
set ytics nomirror font 'Times-Roman, 35'
set nomytics
set ytics 100
#set format y '2^{%T}'
set format y '100^{%L}'
set format x '100^{%L}'
set logscale y 100
set grid noxtics noytics
set grid ytics lw -1
set border 3 lw 4
set key at 0.1, 10000 font 'Times-Roman, 35'
set bmargin 3.5
set pointsize 2.0
set style function linespoints
set style line 1 lw 4 lc rgb '#888888' ps 2 pt 7 pi 4
set style line 2 lt 7 pt 14 lc rgb '#000000'
set style line 3 lc rgb '#A9A9A9' dt 1 lt 1 lw 7 pt 6 pi -1 ps 3.0
set style line 1 lc rgb 'orange' dt 1 lt 1 lw 9 pt 7 pi -1 ps 3.5
set style line 2 lc rgb 'blue' dt 1 lt 1 lw 7 pt 9 pi -1 ps 3.5
set style line 1 lc rgb 'blue' dt 1 lt 1 lw 9 pt 7 pi -1 ps 3.5
set style line 2 lc rgb 'red' dt 2 lt 1 lw 7 pt 9 pi -1 ps 4.5
set style line 3 lc rgb 'black' dt 1 lt 1 lw 7 pt 4 pi -1 ps 3.0
plot \
'dat/${EXPNAME}.dat' using (\$1/100):(\$2) with linespoints ls 1 title 'Aero',\
'dat/${EXPNAME}.dat' using (\$1/100):(\$3) with linespoints ls 3 title 'Aero w/o Freshness',\
'dat/${EXPNAME}.dat' using (\$1/100):(\$4) with linespoints ls 2 title 'Orchard',\
" > plot.plt
gnuplot plot.plt
epspdf $EPS
rm $EPS
