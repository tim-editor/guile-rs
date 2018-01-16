#include <libguile.h>
#include <stdbool.h>

// manual fixes...

#define gen_macro_proxy_0(r, n) \
    r gu_##n() {                \
        return n;               \
    }                           \

#define gen_macro_proxy(r, n, t) \
    r gu_##n (t x) {             \
        return n(x);             \
    }                            \

#define gen_macro_proxy_2(r, n, t1, t2) \
    r gu_##n (t1 x, t2 y) {            \
        return n(x, y);                 \
    }                                   \

gen_macro_proxy(scm_t_bits, SCM_UNPACK, SCM);
gen_macro_proxy_2(bool, scm_is_eq, SCM, SCM);

gen_macro_proxy(bool, scm_is_false, SCM);
gen_macro_proxy(bool, scm_is_true, SCM);

gen_macro_proxy(int, scm_is_symbol, SCM);

gen_macro_proxy(int, scm_is_pair, SCM);
gen_macro_proxy(SCM, scm_car, SCM);
gen_macro_proxy(SCM, scm_cdr, SCM);
gen_macro_proxy_2(SCM, scm_cons, SCM, SCM);

gen_macro_proxy_0(SCM, SCM_BOOL_F);
gen_macro_proxy_0(SCM, SCM_BOOL_T);
gen_macro_proxy_0(SCM, SCM_UNDEFINED);

SCM gu_scm_list_n(SCM* elts) {
    SCM answer = SCM_EOL;
    SCM *pos = &answer;

    while (!SCM_UNBNDP(*elts)) {
        *pos = scm_cons(*elts++, SCM_EOL);
        pos = SCM_CDRLOC(*pos);
    }
    return answer;
}

int gu_scm_is_string(SCM);

int gu_scm_is_string(SCM x) {
    return scm_is_string(x);
}

void test_func() {
    return;
}
