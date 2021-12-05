
set terminal postscript eps 'Times-Roman,30' color 
#set terminal postscript eps 'Times-Roman,30' enhanced monochrome size 6.3,4
set output 'graphs/client_network_upload.eps'
set xrange [ 4096 : 65536]
set xtics nomirror font 'Times-Roman, 44' offset 0,-0.1
set nomxtics
set format x '2^{%L}'
set logscale x 2
set xlabel "Number of users" font 'Times-Roman,44'
set yrange [ 100 : 100000000]
set ylabel "Network transfers (KB)\n " font 'Times-Roman,44' offset -2.0,0.0
set ytics nomirror font 'Times-Roman, 44'
set nomytics
set format y '10^{%T}'
set logscale y 10
set grid noxtics noytics
set border 3 lw 4
set key top left maxrows 3 samplen 3 font 'Times-Roman,30'
set bmargin 3.5
set pointsize 2.0
set size 1.25, 1
set style function linespoints
set style line 1 lw 4 lc rgb '#990042' ps 2 pt 6 pi 5
set style line 2 lw 3 lc rgb '#31f120' ps 2 pt 12 pi 3
set style line 3 lw 3 lc rgb '#0044a5' ps 2 pt 9 pi 5
set style line 4 lw 4 lc rgb '#888888' ps 2 pt 7 pi 4
set style line 6 lt 7 pt 14 lc rgb '#000000'
set style line 7 lt 7 pt 14 lc rgb '#736f6e'
set style line 1 lc rgb 'red'    dt 1 lw 7 lt 2 pt 64 ps 2.5 pi -1 
set style line 2 lc rgb 'blue'   dt 2 lw 7 lt 1 pt 1  ps 3 pi -1
set style line 3 lc rgb 'yellow' dt 3 lw 7 lt 7 pt 5  ps 2.5 pi -1
set style line 4 lc rgb 'brown'  dt 4 lw 7 lt 2 pt 65 ps 2.5 pi -1
set style line 5 lc rgb 'black'  dt 5 lw 7 lt 1 pt 2  ps 3   pi -1
plot 'dat/client_network_upload.dat' using 1:($2/1024) with linespoints ls 1 title 'Addra','dat/client_network_upload.dat' using 1:($3/1024) with linespoints ls 2 title 'P-XPIR (d=1)','dat/client_network_upload.dat' using 1:($4/1024) with linespoints ls 3 title 'P-XPIR (d=2)','dat/client_network_upload.dat' using 1:($5/1024) with linespoints ls 4 title 'P-SealPIR (d=1)','dat/client_network_upload.dat' using 1:($6/1024) with linespoints ls 5 title 'P-SealPIR (d=2)'

