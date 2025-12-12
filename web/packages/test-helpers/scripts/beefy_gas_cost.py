#!/usr/bin/env python3

import matplotlib.pyplot as plt
import numpy as np
# X axis parameter:
xaxis = np.array([28,29,30,31,32,33,34,35,36,37,38,39,40,41])
# Y axis parameter:
yaxis = np.array([797277,814124,830495,845298,861728,879014,895417,913743,930600,948294,963144,980129,997395,1014269])

plt.figure(figsize=(10,6))
plt.plot(xaxis, yaxis)
plt.xlabel('Signature count')
plt.ylabel('Gas cost')
plt.title("Beefy submitFinal gas cost by sample signature count")
plt.show()
