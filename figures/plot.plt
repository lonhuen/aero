
set terminal postscript eps 'Times-Roman,30' color 
set terminal postscript eps 'Times-Roman,30' color size 6.3,4
set output 'graphs/verifier_network_vs_model_size.eps'
set xrange [ 1: 1000 ]
set xtics nomirror font 'Times-Roman, 45' offset 0,-0.1
set nomxtics
set logscale x 10
#set yrange [ 0 : 3000 ]
set xlabel "\# of params (in multiples of 2^{12})" font 'Times-Roman,45'
set ylabel "Network (MiB)\n " font 'Times-Roman,45' offset -1,0.0
set ytics nomirror font 'Times-Roman, 45'
set nomytics
#set ytics 40
#set format y '2^{%T}'
set format y '10^{%L}'
set logscale y 10
set grid noxtics noytics
set grid ytics lw -1
set border 3 lw 4
set bmargin 3.5
set key top left font 'Times-Roman, 45'
set pointsize 2.0
set style function linespoints
set style line 1 lw 4 lc rgb '#888888' ps 2 pt 7 pi 4
set style line 2 lt 7 pt 14 lc rgb '#000000'
set style line 1 lc rgb 'blue' dt 1 lt 1 lw 9 pt 7 pi -1 ps 4.5
set style line 2 lc rgb 'red' dt 1 lt 1 lw 7 pt 9 pi -1 ps 5.5
set style line 1 lc rgb 'blue' dt 1 lt 1 lw 9 pt 7 ps 4.5
set style line 2 lc rgb 'red' dt 2 lt 2 lw 7 pt 9 ps 6.5
plot 'dat/verifier_network_vs_model_size.dat' using 1:($2/1000000) with linespoints ls 1 title 'Aero','dat/verifier_network_vs_model_size.dat' using 1:($3/1000000) with linespoints ls 2 title 'Orchard',
