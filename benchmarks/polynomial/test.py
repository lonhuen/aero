def eval(n, p):
    s = 0
    t = 1
    for x in p:
        s = s + x * t
        t = t * n
    s = s % (127**3)
    if s < 127:
        print(s)
    else:
        print(s - 127**3)

with open("temp.input") as f:
    l = f.readline()
    p = [int(t) for t in l.strip().split(' ') ]
    #for i in range(127):
    #    eval(i,p) 
    #eval(127**3-2,p) 
    eval(1,p) 
    eval(1.5,p) 
    eval(2,p) 
