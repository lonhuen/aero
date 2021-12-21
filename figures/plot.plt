
set terminal postscript eps 'Times-Roman,30' color 
set terminal postscript eps 'Times-Roman,30' color size 6.3,4
set output 'graphs/committee_network_vs_model_size.eps'
set xrange [ 1: 1000 ]
set xtics nomirror font 'Times-Roman, 39' offset 0,-0.1
set nomxtics
set logscale x 10
set xlabel "# of CTs" font 'Times-Roman,39'
set yrange [ 0.01 : 20637 ]
set ylabel "Committee Network (GB)\n " font 'Times-Roman,39' offset -0.0,0.0
set ytics nomirror font 'Times-Roman, 39'
set nomytics
set ytics 10
#set format y '2^{%T}'
#set format y '%E'
set logscale y 10
set grid noxtics noytics
set grid ytics lw -1
set border 3 lw 4
set key top left
set bmargin 3.5
set pointsize 2.0
set style function linespoints
set style line 1 lw 4 lc rgb '#888888' ps 2 pt 7 pi 4
set style line 2 lt 7 pt 14 lc rgb '#000000'
set style line 1 lc rgb 'orange' dt 1 lt 1 lw 9 pt 7 pi -1 ps 3.5
set style line 2 lc rgb 'blue' dt 1 lt 1 lw 7 pt 9 pi -1 ps 3.5
plot 'dat/committee_network_vs_model_size.dat' using 1:($2/1000000000) with linespoints ls 1 title 'Aero','dat/committee_network_vs_model_size.dat' using 1:($3/1000000000) with linespoints ls 2 title 'Baseline',
