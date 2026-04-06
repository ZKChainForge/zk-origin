pragma circom 2.1.0;

template IsZero() {
    signal input in;
    signal output out;
    signal inv;
    inv <-- in != 0 ? 1 / in : 0;
    out <-- in == 0 ? 1 : 0;
    in * inv + out === 1;
    out * (out - 1) === 0;
}

template IsEqual() {
    signal input in[2];
    signal output out;
    component isz = IsZero();
    isz.in <== in[1] - in[0];
    out <== isz.out;
}

template Num2Bits(n) {
    signal input in;
    signal output out[n];
    var lc1 = 0;
    var e2 = 1;
    for (var i = 0; i < n; i++) {
        out[i] <-- (in >> i) & 1;
        out[i] * (out[i] - 1) === 0;
        lc1 += out[i] * e2;
        e2 = e2 + e2;
    }
    lc1 === in;
}

template LessThan(n) {
    assert(n <= 252);
    signal input in[2];
    signal output out;
    component n2b = Num2Bits(n+1);
    n2b.in <== in[0] + (1<<n) - in[1];
    out <== 1 - n2b.out[n];
}

template GreaterThan(n) {
    signal input in[2];
    signal output out;
    component lt = LessThan(n);
    lt.in[0] <== in[1];
    lt.in[1] <== in[0];
    out <== lt.out;
}

template GreaterEqThan(n) {
    signal input in[2];
    signal output out;
    component lt = LessThan(n);
    lt.in[0] <== in[1];
    lt.in[1] <== in[0] + 1;
    out <== lt.out;
}

template LessEqThan(n) {
    signal input in[2];
    signal output out;
    component lt = LessThan(n);
    lt.in[0] <== in[0];
    lt.in[1] <== in[1] + 1;
    out <== lt.out;
}

template Mux1() {
    signal input c[2];
    signal input s;
    signal output out;
    out <== c[0] + s * (c[1] - c[0]);
}

template InRange(n) {
    signal input value;
    signal input min;
    signal input max;
    signal output out;
    component gtEq = GreaterEqThan(n);
    gtEq.in[0] <== value;
    gtEq.in[1] <== min;
    component ltEq = LessEqThan(n);
    ltEq.in[0] <== value;
    ltEq.in[1] <== max;
    out <== gtEq.out * ltEq.out;
}
