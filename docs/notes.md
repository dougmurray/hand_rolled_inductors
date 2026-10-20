# Air-core inductors
## Single layer

Classic ARRL (Wheeler) equation:

  L = (d**2 * n**2) / (18*d + 40*l)

where L is inductance in μH, d is coil diameter in inches, l is coil length in inches, and n is number of turns.

Which can be transposed for inductance based on number of turns:

  n = sqrt((L * (18*d + 40*l)) / d

where, again L is in μH.

The calculated self-capacitance, which is based off the SRF (self-resonance frequency) of an inductor, can be approximated by the Medhurst equation:

  Cs = 0.46 * d

where Cs is in pF, and d is coil diameter in cm. This 0.46 H-factor however is only true for the length/diameter ratio of 1 (l/d = 1). For other situations the H-factor is as follows:

| I/D |   H-factor |
| --- | ---------- |
| 1.0 |   0.46     | 
| 1.5 |   0.47     |
| 2.0 |   0.50     |
| 3.0 |   0.61     |
| 5.0 |   0.81     |


If you want to keep the l/d = 1, then the maximum number of turns you can fit while keeping l/d = 1 is:

  N_max = d / d_wire

where d is the coil diameter in cm, and d_wire is the diameter of the wire in cm.

Finally the SRF (self-resonance frequency) of an inductor is:

  f_SRF = 1 / (2 * π * sqrt(L * C))

Note that if you make an inductor with l/d = 1, then the SRF is also approximately:

  f_SRF = 1 / (n * sqrt(d))

where n is the number of turns, and d is the coil diameter in cm.
