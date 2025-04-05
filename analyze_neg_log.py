with open("result.txt", "w") as g:
    with open("neg_log.ansi", "r") as f:
        while True:
            l = f.readline()
            if 'Received handshake message: Protocol("/multistream/1.0")' in l:
                break
        while (l1 := f.readline()) and (l2 := f.readline()):
            try:
                t1 = float(l1[12:20])
                t2 = float(l2[12:20])
                g.write("{}\n".format(t2 - t1))
            except:
                break
