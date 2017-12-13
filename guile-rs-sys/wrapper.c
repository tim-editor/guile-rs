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

gen_macro_proxy_0(SCM, SCM_BOOL_F);
gen_macro_proxy_0(SCM, SCM_BOOL_T);

/* gen_macro_proxy(SCM, scm_from_intmax, scm_t_intmax); */
/* gen_macro_proxy(SCM, scm_from_uintmax, scm_t_uintmax); */

/* /1* /// <div rustbindgen hide></div> *1/ */
/* static const SCM gu_SCM_BOOL_T = SCM_BOOL_T; */

/* /1* /// <div rustbindgen hide></div> *1/ */
/* static const SCM gu_SCM_BOOL_F = SCM_BOOL_F; */


/* bool gu_scm_is_eq(SCM x, SCM y) { */
/*     return scm_is_eq(x, y); */
/* } */

/* bool gu_scm_is_true(SCM x) { */
/*     return scm_is_true(x); */
/* } */

int gu_scm_is_string(SCM);

int gu_scm_is_string(SCM x) {
    return scm_is_string(x);
}

void test_func() {
    return;
}
