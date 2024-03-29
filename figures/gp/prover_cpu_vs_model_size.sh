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
set xrange [ 1: 1000 ]
set xtics nomirror font 'Times-Roman, 45' offset 0,-0.1
set nomxtics
set logscale x 10
set xlabel \"# of CTs\" font 'Times-Roman,45'
set yrange [ 0 : 256]
set ylabel \"Prover CPU time (min.)\\\n \" font 'Times-Roman,45' offset -0.0,0.0
set ytics nomirror font 'Times-Roman, 45'
set nomytics
#set ytics 40
#set format y '2^{%T}'
#set format y '%E'
set logscale y 2
set grid noxtics noytics
set grid ytics lw -1
set border 3 lw 4
set key top left font 'Times-Roman, 45'
set bmargin 3.2
set pointsize 2.0
set style function linespoints
set style line 1 lw 4 lc rgb 'blue' ps 4 pt 7 pi 4
set style line 2 lt 7 pt 14 lc rgb 'red'
set style line 1 lc rgb 'blue' dt 1 lt 1 lw 9 pt 7 pi -1 ps 4.5
set style line 2 lc rgb 'red' dt 1 lt 1 lw 7 pt 9 pi -1 ps 5.5
plot \
'dat/${EXPNAME}.dat' using 1:(\$2/60) with linespoints ls 1 title 'Aero',\
'dat/${EXPNAME}.dat' using 1:(\$3/60) with linespoints ls 2 title 'Baseline',\
" > plot.plt
gnuplot plot.plt
epspdf $EPS
rm $EPS
