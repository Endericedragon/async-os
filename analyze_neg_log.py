import sys

# with open(sys.argv[1], "w") as g:
#     with open("neg_log.ansi", "r") as f:
#         while l1 := f.readline():
#             if "Suggesting /" in l1:
#                 l2 = f.readline()
#                 try:
#                     t1 = float(l1[12:20])
#                     t2 = float(l2[12:20])
#                     g.write("{}\n".format(t2 - t1))
#                 except:
#                     break
#             else:
#                 continue

with open(sys.argv[1], "w") as g:
    with open("neg_log.ansi", "r") as f:
        while l1 := f.readline():
            if "Negotiation starts at" in l1:
                f.readline()
                l2 = f.readline()
                try:
                    t1 = float(l1[22:])
                    t2 = float(l2[20:])
                    g.write("{}\n".format(t2 - t1))
                except:
                    break
            else:
                continue
