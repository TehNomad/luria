(function () {
  const e = document.createElement("link").relList;
  if (e && e.supports && e.supports("modulepreload")) return;
  for (const n of document.querySelectorAll('link[rel="modulepreload"]')) s(n);
  new MutationObserver((n) => {
    for (const a of n) if (a.type === "childList") for (const i of a.addedNodes) i.tagName === "LINK" && i.rel === "modulepreload" && s(i);
  }).observe(document, { childList: !0, subtree: !0 });
  function t(n) {
    const a = {};
    return (
      n.integrity && (a.integrity = n.integrity),
      n.referrerpolicy && (a.referrerPolicy = n.referrerpolicy),
      n.crossorigin === "use-credentials"
        ? (a.credentials = "include")
        : n.crossorigin === "anonymous"
          ? (a.credentials = "omit")
          : (a.credentials = "same-origin"),
      a
    );
  }
  function s(n) {
    if (n.ep) return;
    n.ep = !0;
    const a = t(n);
    fetch(n.href, a);
  }
})();
function qe() {}
function Ai(l, e) {
  for (const t in e) l[t] = e[t];
  return l;
}
function Qr(l) {
  return l();
}
function Ei() {
  return Object.create(null);
}
function Zt(l) {
  l.forEach(Qr);
}
function Ci(l) {
  return typeof l == "function";
}
function Ll(l, e) {
  return l != l ? e == e : l !== e || (l && typeof l == "object") || typeof l == "function";
}
let $n;
function mn(l, e) {
  return ($n || ($n = document.createElement("a")), ($n.href = e), l === $n.href);
}
function Co(l) {
  return Object.keys(l).length === 0;
}
function xr(l, ...e) {
  if (l == null) return qe;
  const t = l.subscribe(...e);
  return t.unsubscribe ? () => t.unsubscribe() : t;
}
function To(l) {
  let e;
  return (xr(l, (t) => (e = t))(), e);
}
function So(l, e, t, s) {
  if (l) {
    const n = Jr(l, e, t, s);
    return l[0](n);
  }
}
function Jr(l, e, t, s) {
  return l[1] && s ? Ai(t.ctx.slice(), l[1](s(e))) : t.ctx;
}
function ko(l, e, t, s) {
  if (l[2] && s) {
    const n = l[2](s(t));
    if (e.dirty === void 0) return n;
    if (typeof n == "object") {
      const a = [],
        i = Math.max(e.dirty.length, n.length);
      for (let u = 0; u < i; u += 1) a[u] = e.dirty[u] | n[u];
      return a;
    }
    return e.dirty | n;
  }
  return e.dirty;
}
function Mo(l, e, t, s, n, a) {
  if (n) {
    const i = Jr(e, t, s, a);
    l.p(i, n);
  }
}
function Eo(l) {
  if (l.ctx.length > 32) {
    const e = [],
      t = l.ctx.length / 32;
    for (let s = 0; s < t; s++) e[s] = -1;
    return e;
  }
  return -1;
}
function Vs(l) {
  return l == null ? "" : l;
}
function o(l, e) {
  l.appendChild(e);
}
function g(l, e, t) {
  l.insertBefore(e, t || null);
}
function m(l) {
  l.parentNode && l.parentNode.removeChild(l);
}
function cn(l, e) {
  for (let t = 0; t < l.length; t += 1) l[t] && l[t].d(e);
}
function c(l) {
  return document.createElement(l);
}
function C(l) {
  return document.createTextNode(l);
}
function S() {
  return C(" ");
}
function is() {
  return C("");
}
function ce(l, e, t, s) {
  return (l.addEventListener(e, t, s), () => l.removeEventListener(e, t, s));
}
function r(l, e, t) {
  t == null ? l.removeAttribute(e) : l.getAttribute(e) !== t && l.setAttribute(e, t);
}
function Ti(l) {
  let e;
  return {
    p(...t) {
      ((e = t), e.forEach((s) => l.push(s)));
    },
    r() {
      e.forEach((t) => l.splice(l.indexOf(t), 1));
    },
  };
}
function Us(l) {
  return l === "" ? null : +l;
}
function No(l) {
  return Array.from(l.childNodes);
}
function Z(l, e) {
  ((e = "" + e), l.data !== e && (l.data = e));
}
function nt(l, e) {
  l.value = e == null ? "" : e;
}
function Ws(l, e, t, s) {
  t == null ? l.style.removeProperty(e) : l.style.setProperty(e, t, s ? "important" : "");
}
function gn(l, e, t) {
  for (let s = 0; s < l.options.length; s += 1) {
    const n = l.options[s];
    if (n.__value === e) {
      n.selected = !0;
      return;
    }
  }
  (!t || e !== void 0) && (l.selectedIndex = -1);
}
function Gn(l) {
  const e = l.querySelector(":checked");
  return e && e.__value;
}
function ti(l, e, t) {
  l.classList[t ? "add" : "remove"](e);
}
function Po(l, e, { bubbles: t = !1, cancelable: s = !1 } = {}) {
  const n = document.createEvent("CustomEvent");
  return (n.initCustomEvent(l, t, s, e), n);
}
function li(l, e) {
  return new l(e);
}
let In;
function Fn(l) {
  In = l;
}
function si() {
  if (!In) throw new Error("Function called outside component initialization");
  return In;
}
function Zr(l) {
  si().$$.on_mount.push(l);
}
function Lo(l) {
  si().$$.after_update.push(l);
}
function Dn(l) {
  si().$$.on_destroy.push(l);
}
function $r() {
  const l = si();
  return (e, t, { cancelable: s = !1 } = {}) => {
    const n = l.$$.callbacks[e];
    if (n) {
      const a = Po(e, t, { cancelable: s });
      return (
        n.slice().forEach((i) => {
          i.call(l, a);
        }),
        !a.defaultPrevented
      );
    }
    return !0;
  };
}
function qi(l, e) {
  const t = l.$$.callbacks[e.type];
  t && t.slice().forEach((s) => s.call(this, e));
}
const yn = [],
  dt = [];
let Cn = [];
const wi = [],
  eo = Promise.resolve();
let yi = !1;
function to() {
  yi || ((yi = !0), eo.then(so));
}
function lo() {
  return (to(), eo);
}
function vn(l) {
  Cn.push(l);
}
function Wt(l) {
  wi.push(l);
}
const vi = new Set();
let bn = 0;
function so() {
  if (bn !== 0) return;
  const l = In;
  do {
    try {
      for (; bn < yn.length; ) {
        const e = yn[bn];
        (bn++, Fn(e), Go(e.$$));
      }
    } catch (e) {
      throw ((yn.length = 0), (bn = 0), e);
    }
    for (Fn(null), yn.length = 0, bn = 0; dt.length; ) dt.pop()();
    for (let e = 0; e < Cn.length; e += 1) {
      const t = Cn[e];
      vi.has(t) || (vi.add(t), t());
    }
    Cn.length = 0;
  } while (yn.length);
  for (; wi.length; ) wi.pop()();
  ((yi = !1), vi.clear(), Fn(l));
}
function Go(l) {
  if (l.fragment !== null) {
    (l.update(), Zt(l.before_update));
    const e = l.dirty;
    ((l.dirty = [-1]), l.fragment && l.fragment.p(l.ctx, e), l.after_update.forEach(vn));
  }
}
function Fo(l) {
  const e = [],
    t = [];
  (Cn.forEach((s) => (l.indexOf(s) === -1 ? e.push(s) : t.push(s))), t.forEach((s) => s()), (Cn = e));
}
const ei = new Set();
let _n;
function vs() {
  _n = { r: 0, c: [], p: _n };
}
function _s() {
  (_n.r || Zt(_n.c), (_n = _n.p));
}
function ve(l, e) {
  l && l.i && (ei.delete(l), l.i(e));
}
function be(l, e, t, s) {
  if (l && l.o) {
    if (ei.has(l)) return;
    (ei.add(l),
      _n.c.push(() => {
        (ei.delete(l), s && (t && l.d(1), s()));
      }),
      l.o(e));
  } else s && s();
}
function Ro(l, e) {
  (l.d(1), e.delete(l.key));
}
function Io(l, e, t, s, n, a, i, u, f, h, d, p) {
  let v = l.length,
    _ = a.length,
    b = v;
  const w = {};
  for (; b--; ) w[l[b].key] = b;
  const y = [],
    A = new Map(),
    q = new Map(),
    T = [];
  for (b = _; b--; ) {
    const E = p(n, a, b),
      P = t(E);
    let G = i.get(P);
    (G ? s && T.push(() => G.p(E, e)) : ((G = h(P, E)), G.c()), A.set(P, (y[b] = G)), P in w && q.set(P, Math.abs(b - w[P])));
  }
  const N = new Set(),
    M = new Set();
  function k(E) {
    (ve(E, 1), E.m(u, d), i.set(E.key, E), (d = E.first), _--);
  }
  for (; v && _; ) {
    const E = y[_ - 1],
      P = l[v - 1],
      G = E.key,
      O = P.key;
    E === P
      ? ((d = E.first), v--, _--)
      : A.has(O)
        ? !i.has(G) || N.has(G)
          ? k(E)
          : M.has(O)
            ? v--
            : q.get(G) > q.get(O)
              ? (M.add(G), k(E))
              : (N.add(O), v--)
        : (f(P, i), v--);
  }
  for (; v--; ) {
    const E = l[v];
    A.has(E.key) || f(E, i);
  }
  for (; _; ) k(y[_ - 1]);
  return (Zt(T), y);
}
function no(l, e) {
  const t = {},
    s = {},
    n = { $$scope: 1 };
  let a = l.length;
  for (; a--; ) {
    const i = l[a],
      u = e[a];
    if (u) {
      for (const f in i) f in u || (s[f] = 1);
      for (const f in u) n[f] || ((t[f] = u[f]), (n[f] = 1));
      l[a] = u;
    } else for (const f in i) n[f] = 1;
  }
  for (const i in s) i in t || (t[i] = void 0);
  return t;
}
function io(l) {
  return typeof l == "object" && l !== null ? l : {};
}
function zt(l, e, t) {
  const s = l.$$.props[e];
  s !== void 0 && ((l.$$.bound[s] = t), t(l.$$.ctx[s]));
}
function Re(l) {
  l && l.c();
}
function Ge(l, e, t, s) {
  const { fragment: n, after_update: a } = l.$$;
  (n && n.m(e, t),
    s ||
      vn(() => {
        const i = l.$$.on_mount.map(Qr).filter(Ci);
        (l.$$.on_destroy ? l.$$.on_destroy.push(...i) : Zt(i), (l.$$.on_mount = []));
      }),
    a.forEach(vn));
}
function Fe(l, e) {
  const t = l.$$;
  t.fragment !== null &&
    (Fo(t.after_update), Zt(t.on_destroy), t.fragment && t.fragment.d(e), (t.on_destroy = t.fragment = null), (t.ctx = []));
}
function Do(l, e) {
  (l.$$.dirty[0] === -1 && (yn.push(l), to(), l.$$.dirty.fill(0)), (l.$$.dirty[(e / 31) | 0] |= 1 << (e % 31)));
}
function Il(l, e, t, s, n, a, i, u = [-1]) {
  const f = In;
  Fn(l);
  const h = (l.$$ = {
    fragment: null,
    ctx: [],
    props: a,
    update: qe,
    not_equal: n,
    bound: Ei(),
    on_mount: [],
    on_destroy: [],
    on_disconnect: [],
    before_update: [],
    after_update: [],
    context: new Map(e.context || (f ? f.$$.context : [])),
    callbacks: Ei(),
    dirty: u,
    skip_bound: !1,
    root: e.target || f.$$.root,
  });
  i && i(h.root);
  let d = !1;
  if (
    ((h.ctx = t
      ? t(l, e.props || {}, (p, v, ..._) => {
          const b = _.length ? _[0] : v;
          return (h.ctx && n(h.ctx[p], (h.ctx[p] = b)) && (!h.skip_bound && h.bound[p] && h.bound[p](b), d && Do(l, p)), v);
        })
      : []),
    h.update(),
    (d = !0),
    Zt(h.before_update),
    (h.fragment = s ? s(h.ctx) : !1),
    e.target)
  ) {
    if (e.hydrate) {
      const p = No(e.target);
      (h.fragment && h.fragment.l(p), p.forEach(m));
    } else h.fragment && h.fragment.c();
    (e.intro && ve(l.$$.fragment), Ge(l, e.target, e.anchor, e.customElement), so());
  }
  Fn(f);
}
class Dl {
  $destroy() {
    (Fe(this, 1), (this.$destroy = qe));
  }
  $on(e, t) {
    if (!Ci(t)) return qe;
    const s = this.$$.callbacks[e] || (this.$$.callbacks[e] = []);
    return (
      s.push(t),
      () => {
        const n = s.indexOf(t);
        n !== -1 && s.splice(n, 1);
      }
    );
  }
  $set(e) {
    this.$$set && !Co(e) && ((this.$$.skip_bound = !0), this.$$set(e), (this.$$.skip_bound = !1));
  }
}
const qn = [];
function ro(l, e) {
  return { subscribe: Si(l, e).subscribe };
}
function Si(l, e = qe) {
  let t;
  const s = new Set();
  function n(u) {
    if (Ll(l, u) && ((l = u), t)) {
      const f = !qn.length;
      for (const h of s) (h[1](), qn.push(h, l));
      if (f) {
        for (let h = 0; h < qn.length; h += 2) qn[h][0](qn[h + 1]);
        qn.length = 0;
      }
    }
  }
  function a(u) {
    n(u(l));
  }
  function i(u, f = qe) {
    const h = [u, f];
    return (
      s.add(h),
      s.size === 1 && (t = e(n) || qe),
      u(l),
      () => {
        (s.delete(h), s.size === 0 && t && (t(), (t = null)));
      }
    );
  }
  return { set: n, update: a, subscribe: i };
}
function oo(l, e, t) {
  const s = !Array.isArray(l),
    n = s ? [l] : l,
    a = e.length < 2;
  return ro(t, (i) => {
    let u = !1;
    const f = [];
    let h = 0,
      d = qe;
    const p = () => {
        if (h) return;
        d();
        const _ = e(s ? f[0] : f, i);
        a ? i(_) : (d = Ci(_) ? _ : qe);
      },
      v = n.map((_, b) =>
        xr(
          _,
          (w) => {
            ((f[b] = w), (h &= ~(1 << b)), u && p());
          },
          () => {
            h |= 1 << b;
          },
        ),
      );
    return (
      (u = !0),
      p(),
      function () {
        (Zt(v), d(), (u = !1));
      }
    );
  });
}
function Oo(l, e) {
  if (l instanceof RegExp) return { keys: !1, pattern: l };
  var t,
    s,
    n,
    a,
    i = [],
    u = "",
    f = l.split("/");
  for (f[0] || f.shift(); (n = f.shift()); )
    ((t = n[0]),
      t === "*"
        ? (i.push("wild"), (u += "/(.*)"))
        : t === ":"
          ? ((s = n.indexOf("?", 1)),
            (a = n.indexOf(".", 1)),
            i.push(n.substring(1, ~s ? s : ~a ? a : n.length)),
            (u += !!~s && !~a ? "(?:/([^/]+?))?" : "/([^/]+?)"),
            ~a && (u += (~s ? "?" : "") + "\\" + n.substring(a)))
          : (u += "/" + n));
  return { keys: i, pattern: new RegExp("^" + u + (e ? "(?=$|/)" : "/?$"), "i") };
}
function jo(l) {
  let e, t, s;
  const n = [l[2]];
  var a = l[0];
  function i(u) {
    let f = {};
    for (let h = 0; h < n.length; h += 1) f = Ai(f, n[h]);
    return { props: f };
  }
  return (
    a && ((e = li(a, i())), e.$on("routeEvent", l[7])),
    {
      c() {
        (e && Re(e.$$.fragment), (t = is()));
      },
      m(u, f) {
        (e && Ge(e, u, f), g(u, t, f), (s = !0));
      },
      p(u, f) {
        const h = f & 4 ? no(n, [io(u[2])]) : {};
        if (f & 1 && a !== (a = u[0])) {
          if (e) {
            vs();
            const d = e;
            (be(d.$$.fragment, 1, 0, () => {
              Fe(d, 1);
            }),
              _s());
          }
          a ? ((e = li(a, i())), e.$on("routeEvent", u[7]), Re(e.$$.fragment), ve(e.$$.fragment, 1), Ge(e, t.parentNode, t)) : (e = null);
        } else a && e.$set(h);
      },
      i(u) {
        s || (e && ve(e.$$.fragment, u), (s = !0));
      },
      o(u) {
        (e && be(e.$$.fragment, u), (s = !1));
      },
      d(u) {
        (u && m(t), e && Fe(e, u));
      },
    }
  );
}
function Bo(l) {
  let e, t, s;
  const n = [{ params: l[1] }, l[2]];
  var a = l[0];
  function i(u) {
    let f = {};
    for (let h = 0; h < n.length; h += 1) f = Ai(f, n[h]);
    return { props: f };
  }
  return (
    a && ((e = li(a, i())), e.$on("routeEvent", l[6])),
    {
      c() {
        (e && Re(e.$$.fragment), (t = is()));
      },
      m(u, f) {
        (e && Ge(e, u, f), g(u, t, f), (s = !0));
      },
      p(u, f) {
        const h = f & 6 ? no(n, [f & 2 && { params: u[1] }, f & 4 && io(u[2])]) : {};
        if (f & 1 && a !== (a = u[0])) {
          if (e) {
            vs();
            const d = e;
            (be(d.$$.fragment, 1, 0, () => {
              Fe(d, 1);
            }),
              _s());
          }
          a ? ((e = li(a, i())), e.$on("routeEvent", u[6]), Re(e.$$.fragment), ve(e.$$.fragment, 1), Ge(e, t.parentNode, t)) : (e = null);
        } else a && e.$set(h);
      },
      i(u) {
        s || (e && ve(e.$$.fragment, u), (s = !0));
      },
      o(u) {
        (e && be(e.$$.fragment, u), (s = !1));
      },
      d(u) {
        (u && m(t), e && Fe(e, u));
      },
    }
  );
}
function Ho(l) {
  let e, t, s, n;
  const a = [Bo, jo],
    i = [];
  function u(f, h) {
    return f[1] ? 0 : 1;
  }
  return (
    (e = u(l)),
    (t = i[e] = a[e](l)),
    {
      c() {
        (t.c(), (s = is()));
      },
      m(f, h) {
        (i[e].m(f, h), g(f, s, h), (n = !0));
      },
      p(f, [h]) {
        let d = e;
        ((e = u(f)),
          e === d
            ? i[e].p(f, h)
            : (vs(),
              be(i[d], 1, 1, () => {
                i[d] = null;
              }),
              _s(),
              (t = i[e]),
              t ? t.p(f, h) : ((t = i[e] = a[e](f)), t.c()),
              ve(t, 1),
              t.m(s.parentNode, s)));
      },
      i(f) {
        n || (ve(t), (n = !0));
      },
      o(f) {
        (be(t), (n = !1));
      },
      d(f) {
        (i[e].d(f), f && m(s));
      },
    }
  );
}
function Ni() {
  const l = window.location.href.indexOf("#/");
  let e = l > -1 ? window.location.href.substr(l + 1) : "/";
  const t = e.indexOf("?");
  let s = "";
  return (t > -1 && ((s = e.substr(t + 1)), (e = e.substr(0, t))), { location: e, querystring: s });
}
const ki = ro(null, function (e) {
  e(Ni());
  const t = () => {
    e(Ni());
  };
  return (
    window.addEventListener("hashchange", t, !1),
    function () {
      window.removeEventListener("hashchange", t, !1);
    }
  );
});
oo(ki, (l) => l.location);
oo(ki, (l) => l.querystring);
const Pi = Si(void 0);
function Uo(l) {
  l ? window.scrollTo(l.__svelte_spa_router_scrollX, l.__svelte_spa_router_scrollY) : window.scrollTo(0, 0);
}
function Vo(l, e, t) {
  let { routes: s = {} } = e,
    { prefix: n = "" } = e,
    { restoreScrollState: a = !1 } = e;
  class i {
    constructor(M, k) {
      if (!k || (typeof k != "function" && (typeof k != "object" || k._sveltesparouter !== !0))) throw Error("Invalid component object");
      if (
        !M ||
        (typeof M == "string" && (M.length < 1 || (M.charAt(0) != "/" && M.charAt(0) != "*"))) ||
        (typeof M == "object" && !(M instanceof RegExp))
      )
        throw Error('Invalid value for "path" argument - strings must start with / or *');
      const { pattern: E, keys: P } = Oo(M);
      ((this.path = M),
        typeof k == "object" && k._sveltesparouter === !0
          ? ((this.component = k.component),
            (this.conditions = k.conditions || []),
            (this.userData = k.userData),
            (this.props = k.props || {}))
          : ((this.component = () => Promise.resolve(k)), (this.conditions = []), (this.props = {})),
        (this._pattern = E),
        (this._keys = P));
    }
    match(M) {
      if (n) {
        if (typeof n == "string")
          if (M.startsWith(n)) M = M.substr(n.length) || "/";
          else return null;
        else if (n instanceof RegExp) {
          const G = M.match(n);
          if (G && G[0]) M = M.substr(G[0].length) || "/";
          else return null;
        }
      }
      const k = this._pattern.exec(M);
      if (k === null) return null;
      if (this._keys === !1) return k;
      const E = {};
      let P = 0;
      for (; P < this._keys.length; ) {
        try {
          E[this._keys[P]] = decodeURIComponent(k[P + 1] || "") || null;
        } catch {
          E[this._keys[P]] = null;
        }
        P++;
      }
      return E;
    }
    async checkConditions(M) {
      for (let k = 0; k < this.conditions.length; k++) if (!(await this.conditions[k](M))) return !1;
      return !0;
    }
  }
  const u = [];
  s instanceof Map
    ? s.forEach((N, M) => {
        u.push(new i(M, N));
      })
    : Object.keys(s).forEach((N) => {
        u.push(new i(N, s[N]));
      });
  let f = null,
    h = null,
    d = {};
  const p = $r();
  async function v(N, M) {
    (await lo(), p(N, M));
  }
  let _ = null,
    b = null;
  a &&
    ((b = (N) => {
      N.state && (N.state.__svelte_spa_router_scrollY || N.state.__svelte_spa_router_scrollX) ? (_ = N.state) : (_ = null);
    }),
    window.addEventListener("popstate", b),
    Lo(() => {
      Uo(_);
    }));
  let w = null,
    y = null;
  const A = ki.subscribe(async (N) => {
    w = N;
    let M = 0;
    for (; M < u.length; ) {
      const k = u[M].match(N.location);
      if (!k) {
        M++;
        continue;
      }
      const E = {
        route: u[M].path,
        location: N.location,
        querystring: N.querystring,
        userData: u[M].userData,
        params: k && typeof k == "object" && Object.keys(k).length ? k : null,
      };
      if (!(await u[M].checkConditions(E))) {
        (t(0, (f = null)), (y = null), v("conditionsFailed", E));
        return;
      }
      v("routeLoading", Object.assign({}, E));
      const P = u[M].component;
      if (y != P) {
        P.loading
          ? (t(0, (f = P.loading)),
            (y = P),
            t(1, (h = P.loadingParams)),
            t(2, (d = {})),
            v("routeLoaded", Object.assign({}, E, { component: f, name: f.name, params: h })))
          : (t(0, (f = null)), (y = null));
        const G = await P();
        if (N != w) return;
        (t(0, (f = (G && G.default) || G)), (y = P));
      }
      (k && typeof k == "object" && Object.keys(k).length ? t(1, (h = k)) : t(1, (h = null)),
        t(2, (d = u[M].props)),
        v("routeLoaded", Object.assign({}, E, { component: f, name: f.name, params: h })).then(() => {
          Pi.set(h);
        }));
      return;
    }
    (t(0, (f = null)), (y = null), Pi.set(void 0));
  });
  Dn(() => {
    (A(), b && window.removeEventListener("popstate", b));
  });
  function q(N) {
    qi.call(this, l, N);
  }
  function T(N) {
    qi.call(this, l, N);
  }
  return (
    (l.$$set = (N) => {
      ("routes" in N && t(3, (s = N.routes)),
        "prefix" in N && t(4, (n = N.prefix)),
        "restoreScrollState" in N && t(5, (a = N.restoreScrollState)));
    }),
    (l.$$.update = () => {
      l.$$.dirty & 32 && (history.scrollRestoration = a ? "manual" : "auto");
    }),
    [f, h, d, s, n, a, q, T]
  );
}
class Ko extends Dl {
  constructor(e) {
    (super(), Il(this, e, Vo, Ho, Ll, { routes: 3, prefix: 4, restoreScrollState: 5 }));
  }
}
var ao = {};
(function (l) {
  var e = l;
  return (
    (e.isNumeric = function (t) {
      var s;
      return ((s = t / 1), !isNaN(s));
    }),
    (e.trim = function (s) {
      s = s.replace(/^\s+/, "");
      for (var n = s.length - 1; n >= 0; n--)
        if (/\S/.test(s.charAt(n))) {
          s = s.slice(0, n + 1);
          break;
        }
      return s;
    }),
    (e.DEGENERATES = {
      A: "A",
      C: "C",
      G: "G",
      T: "T",
      U: "U",
      W: "AT",
      S: "CG",
      M: "AC",
      K: "GT",
      R: "AG",
      Y: "CT",
      B: "CGT",
      D: "AGT",
      H: "ACT",
      V: "ACG",
      N: "ACGT",
      a: "a",
      c: "c",
      g: "g",
      t: "t",
      u: "u",
      w: "at",
      s: "cg",
      m: "ac",
      k: "gt",
      r: "ag",
      y: "ct",
      b: "cgt",
      d: "agt",
      h: "act",
      v: "acg",
      n: "acgt",
    }),
    (e.areEqualDNABases = function (t, s) {
      if (!t || !s) return !1;
      var n = e.DEGENERATES[t];
      n = n.toLowerCase();
      var a = e.DEGENERATES[s];
      a = a.toLowerCase();
      for (var i = n.length; i > -1; --i) if (a.indexOf(n[i]) !== -1) return !0;
      return !1;
    }),
    (e.COMPLEMENTS = (function () {
      var t,
        s = "ACGTUNSWMKRYVDHBacgtunswmkryvdhb -",
        n = "TGCAANSWKMYRBHDVtgcaanswkmyrbhdv -",
        a = {};
      for (t = 0; t < s.length; ++t) a[s.charAt(t)] = n.charAt(t);
      return a;
    })()),
    (e.AMINO_ACID_TLC = {
      ALA: "A",
      ASX: "B",
      CYS: "C",
      ASP: "D",
      GLU: "E",
      PHE: "F",
      GLY: "G",
      HIS: "H",
      XLE: "J",
      ILE: "I",
      LYS: "K",
      LEU: "L",
      MET: "M",
      ASN: "N",
      PYL: "O",
      PRO: "P",
      GLN: "Q",
      ARG: "R",
      SER: "S",
      THR: "T",
      SEC: "U",
      VAL: "V",
      TRP: "W",
      ALL: "X",
      TYR: "Y",
      GLX: "Z",
      TER: "*",
    }),
    (e.AMINO_ACID_OLC = {
      A: "Ala",
      B: "Asx",
      C: "Cys",
      D: "Asp",
      E: "Glu",
      F: "Phe",
      G: "Gly",
      H: "His",
      I: "Ile",
      J: "Xle",
      K: "Lys",
      L: "Leu",
      M: "Met",
      N: "Asn",
      O: "Pyl",
      P: "Pro",
      Q: "Gln",
      R: "Arg",
      S: "Ser",
      T: "Thr",
      U: "Sec",
      V: "Val",
      W: "Trp",
      X: "All",
      Y: "Tyr",
      Z: "Glx",
      "*": "Ter",
    }),
    (e.validAARegex = (function () {
      return /([ABCDEFGHIKLMNOPQRSTUVWXYZ\*])/gi;
    })()),
    (e.invalidAARegex = (function () {
      return /([^ABCDEFGHIKLMNOPQRSTUVWXYZ\*])/gi;
    })()),
    (e.validNARegex = (function () {
      return /([ACGTUNSWMKRYVDHBacgtunswmkryvdhb])/g;
    })()),
    (e.invalidNARegex = (function () {
      return /([^ACGTUNSWMKRYVDHBacgtunswmkryvdhb])/g;
    })()),
    (e.validNAStrictRegex = (function () {
      return /([TCAGtcag])/g;
    })()),
    (e.invalidNAStrictRegex = (function () {
      return /([^TCAGtcag])/g;
    })()),
    (e.isValidAASeq = function (t) {
      return ((e.invalidAARegex.lastIndex = 0), !e.invalidAARegex.exec(t));
    }),
    (e.isValidNASeq = function (t, s) {
      s = s || !1;
      var n = s ? e.invalidNARegex : e.invalidNAStrictRegex;
      return ((n.lastIndex = 0), !n.exec(t));
    }),
    (e.aaOneToThree = function (t) {
      for (var s = t.split(""), n = [], a = 0; a < s.length; a++)
        if (e.AMINO_ACID_OLC[s[a]]) n.push(e.AMINO_ACID_OLC[s[a]]);
        else throw "Unknown one-letter-code: " + s[a];
      return n.join("");
    }),
    (e.CODONS = (function () {
      var t,
        s,
        n,
        a = [],
        i = ["T", "C", "A", "G"],
        u,
        f,
        h;
      for (u = 0; u < 4; ++u) for (t = i[u], f = 0; f < 4; ++f) for (s = i[f], h = 0; h < 4; ++h) ((n = i[h]), a.push(t + s + n));
      return a;
    })()),
    (e.CODON_TABLE_DATA = [
      `1
FFLLSSSSYY**CC*WLLLLPPPPHHQQRRRRIIIMTTTTNNKKSSRRVVVVAAAADDEEGGGG
---M---------------M---------------M----------------------------`,
      `2
FFLLSSSSYY**CCWWLLLLPPPPHHQQRRRRIIMMTTTTNNKKSS**VVVVAAAADDEEGGGG
--------------------------------MMMM---------------M------------`,
      `3
FFLLSSSSYY**CCWWTTTTPPPPHHQQRRRRIIMMTTTTNNKKSSRRVVVVAAAADDEEGGGG
----------------------------------MM----------------------------`,
      `4
FFLLSSSSYY**CCWWLLLLPPPPHHQQRRRRIIIMTTTTNNKKSSRRVVVVAAAADDEEGGGG
--MM---------------M------------MMMM---------------M------------`,
      `5
FFLLSSSSYY**CCWWLLLLPPPPHHQQRRRRIIMMTTTTNNKKSSSSVVVVAAAADDEEGGGG
---M----------------------------MMMM---------------M------------`,
      `6
FFLLSSSSYYQQCC*WLLLLPPPPHHQQRRRRIIIMTTTTNNKKSSRRVVVVAAAADDEEGGGG
-----------------------------------M----------------------------`,
      `9
FFLLSSSSYY**CCWWLLLLPPPPHHQQRRRRIIIMTTTTNNNKSSSSVVVVAAAADDEEGGGG
-----------------------------------M---------------M------------`,
      `10
FFLLSSSSYY**CC*WLLLLPPPPHHQQRRRRIIIMTTTTNNKKSSRRVVVVAAAADDEEGGGG
---M---------------M------------MMMM---------------M------------`,
      `11
FFLLSSSSYY**CC*WLLLLPPPPHHQQRRRRIIIMTTTTNNKKSSRRVVVVAAAADDEEGGGG
---M---------------M------------MMMM---------------M------------`,
      `12
FFLLSSSSYY**CC*WLLLSPPPPHHQQRRRRIIIMTTTTNNKKSSRRVVVVAAAADDEEGGGG
-------------------M---------------M----------------------------`,
      `13
FFLLSSSSYY**CCWWLLLLPPPPHHQQRRRRIIMMTTTTNNKKSSGGVVVVAAAADDEEGGGG
---M------------------------------MM---------------M------------`,
      `14
FFLLSSSSYYY*CCWWLLLLPPPPHHQQRRRRIIIMTTTTNNNKSSSSVVVVAAAADDEEGGGG
-----------------------------------M----------------------------`,
      `15
FFLLSSSSYY*QCC*WLLLLPPPPHHQQRRRRIIIMTTTTNNKKSSRRVVVVAAAADDEEGGGG
-----------------------------------M----------------------------`,
      `16
FFLLSSSSYY*LCC*WLLLLPPPPHHQQRRRRIIIMTTTTNNKKSSRRVVVVAAAADDEEGGGG
-----------------------------------M----------------------------`,
      `21
FFLLSSSSYY**CCWWLLLLPPPPHHQQRRRRIIMMTTTTNNNKSSSSVVVVAAAADDEEGGGG
-----------------------------------M---------------M------------`,
      `22
FFLLSS*SYY*LCC*WLLLLPPPPHHQQRRRRIIIMTTTTNNKKSSRRVVVVAAAADDEEGGGG
-----------------------------------M----------------------------`,
      `23
FF*LSSSSYY**CC*WLLLLPPPPHHQQRRRRIIIMTTTTNNKKSSRRVVVVAAAADDEEGGGG
--------------------------------M--M---------------M------------`,
      `24
FFLLSSSSYY**CCWWLLLLPPPPHHQQRRRRIIIMTTTTNNKKSSSKVVVVAAAADDEEGGGG
---M---------------M---------------M---------------M------------`,
      `25
FFLLSSSSYY**CCGWLLLLPPPPHHQQRRRRIIIMTTTTNNKKSSRRVVVVAAAADDEEGGGG
---M-------------------------------M---------------M------------`,
      `90
FFLLSSSSYY*QCC*WLLLLPPPPHHQQRRRRIIIMTTTTNNKKSSRRVVVVAAAADDEEGGGG
---M---------------M------------MMMM---------------M------------`,
    ]),
    (e.cut = {
      Ecoli_K12: [
        ["Ala", "GCG", 0.36],
        ["Ala", "GCC", 0.27],
        ["Ala", "GCA", 0.21],
        ["Ala", "GCT", 0.16],
        ["Arg", "CGC", 0.4],
        ["Arg", "CGT", 0.38],
        ["Arg", "CGG", 0.1],
        ["Arg", "CGA", 0.06],
        ["Arg", "AGA", 0.04],
        ["Arg", "AGG", 0.02],
        ["Asn", "AAC", 0.55],
        ["Asn", "AAT", 0.45],
        ["Asp", "GAT", 0.63],
        ["Asp", "GAC", 0.37],
        ["Cys", "TGC", 0.55],
        ["Cys", "TGT", 0.45],
        ["Gln", "CAG", 0.65],
        ["Gln", "CAA", 0.35],
        ["Glu", "GAA", 0.69],
        ["Glu", "GAG", 0.31],
        ["Gly", "GGC", 0.4],
        ["Gly", "GGT", 0.34],
        ["Gly", "GGG", 0.15],
        ["Gly", "GGA", 0.11],
        ["His", "CAT", 0.57],
        ["His", "CAC", 0.43],
        ["Ile", "ATT", 0.51],
        ["Ile", "ATC", 0.42],
        ["Ile", "ATA", 0.07],
        ["Leu", "CTG", 0.5],
        ["Leu", "TTG", 0.13],
        ["Leu", "TTA", 0.13],
        ["Leu", "CTT", 0.1],
        ["Leu", "CTC", 0.1],
        ["Leu", "CTA", 0.04],
        ["Lys", "AAA", 0.77],
        ["Lys", "AAG", 0.23],
        ["Met", "ATG", 1],
        ["Phe", "TTT", 0.57],
        ["Phe", "TTC", 0.43],
        ["Pro", "CCG", 0.52],
        ["Pro", "CCA", 0.19],
        ["Pro", "CCT", 0.16],
        ["Pro", "CCC", 0.12],
        ["Ser", "AGC", 0.28],
        ["Ser", "AGT", 0.15],
        ["Ser", "TCG", 0.15],
        ["Ser", "TCT", 0.15],
        ["Ser", "TCC", 0.15],
        ["Ser", "TCA", 0.12],
        ["Thr", "ACC", 0.44],
        ["Thr", "ACG", 0.27],
        ["Thr", "ACT", 0.17],
        ["Thr", "ACA", 0.13],
        ["Trp", "TGG", 1],
        ["Tyr", "TAT", 0.57],
        ["Tyr", "TAC", 0.43],
        ["Val", "GTG", 0.37],
        ["Val", "GTT", 0.26],
        ["Val", "GTC", 0.22],
        ["Val", "GTA", 0.15],
        ["Ter", "TAA", 0.64],
        ["Ter", "TGA", 0.29],
        ["Ter", "TAG", 0.07],
        ["All", "NNK", 1],
      ],
    }),
    (e.sanitizeSeq = function (t) {
      return t.replace(/[\s\d\/]+/gi, "");
    }),
    (e.revcomp = function (t, s) {
      if (((t = t || ""), t === "")) return "";
      s == null ? (s = !0) : (s = !!s);
      for (var n = t.split(""), a = 0; a < n.length; ++a)
        if (e.COMPLEMENTS[n[a]]) n[a] = e.COMPLEMENTS[n[a]];
        else if (s) throw "Unknown base: " + n[a];
      for (var i = "", u = n.length - 1; u >= 0; u--) i += n[u];
      return i;
    }),
    (e.rev = function (t) {
      if (((t = t || ""), t === "")) return "";
      for (var s = t.split(""), n = "", a = s.length - 1; a >= 0; a--) n += s[a];
      return n;
    }),
    (e.comp = function (t, s) {
      if (((t = t || ""), t === "")) return "";
      (s === "undefined" || s === null) && (s = !0);
      for (var n = t.split(""), a = 0; a < n.length; ++a)
        if (e.COMPLEMENTS[n[a]]) n[a] = e.COMPLEMENTS[n[a]];
        else if (s) throw "Unknown base: " + n[a];
      return n.join("");
    }),
    (e.TranslationTable = function (t, s) {
      ((this.self = this),
        (this.codons = t),
        (this.starts = s),
        (this.translate = function (n, a) {
          if (n.length !== 3) throw "Invalid codon: " + n;
          var i = n.split("").map(function (v) {
              if (!v) return [];
              var _ = e.DEGENERATES[v];
              return _ ? e.DEGENERATES[v].split("") : [];
            }),
            u = (function () {
              var v,
                _,
                b,
                w = [],
                y,
                A,
                q,
                T;
              for (A = 0; A < i[0].length; ++A) {
                for (y = [], v = i[0][A], q = 0; q < i[1].length; ++q)
                  for (_ = i[1][q], T = 0; T < i[2].length; ++T) ((b = i[2][T]), y.push(v + _ + b));
                w.push(y);
              }
              return w;
            })(),
            f = (function () {
              for (
                var v = [],
                  _ = function (w) {
                    t[w] ? v.push(t[w]) : v.push("X");
                  },
                  b = 0;
                b < u.length;
                b++
              )
                u[b].forEach(_);
              return v;
            })(),
            h = (function () {
              for (
                var v = [],
                  _ = function (w) {
                    s[w] ? v.push(s[w]) : v.push(!1);
                  },
                  b = 0;
                b < u.length;
                b++
              )
                u[b].forEach(_);
              return v;
            })(),
            d;
          ((f = Array.from(new Set(f))),
            f.length === 1
              ? (d = f.pop())
              : f.length === 2
                ? f.indexOf("D") + f.indexOf("N") === 1
                  ? (d = "B")
                  : f.indexOf("E") + f.indexOf("Q") === 1
                    ? (d = "Z")
                    : f.indexOf("I") + f.indexOf("L") === 1
                      ? (d = "J")
                      : (d = "X")
                : (d = "X"));
          var p;
          return (h.length > 1 ? (p = !1) : (p = h.pop()), [d, p]);
        }));
    }),
    (e.buildhash = function (t, s) {
      var n = {};
      if (t.length !== s.length) throw "Arrays for buildhash must be equal length";
      for (var a = 0; a < t.length; a++) n[t[a].toString()] = s[a];
      return n;
    }),
    (e.CODON_TABLES = (function () {
      var t = {};
      return (
        e.CODON_TABLE_DATA.forEach(function (s) {
          var n = s.split(`
`);
          if (n.length !== 3) throw "codon table not parsed properly";
          var a = n[0],
            i = e.buildhash(e.CODONS, n[1].split("")),
            u = e.buildhash(
              e.CODONS,
              n[2].split("").map(function (f) {
                return f === "M";
              }),
            );
          t[a] = new e.TranslationTable(i, u);
        }),
        t
      );
    })()),
    (e.translate = function (t, s, n, a, i) {
      if (((n = n || "1"), (s = s || 0), (a = a || []), (i = i || [!0, !0]), !t || t.length === 0)) return "";
      if (((t = t.slice(s).toUpperCase().replace(/U/g, "T")), !e.CODON_TABLES[n])) throw "Unknown translation table: " + n;
      if (t.length < 3) throw "Sequence length is too short.";
      var u = e.CODON_TABLES[n],
        f = u.translate(t.slice(0, 3), t.length === 3),
        h = f[0],
        d = f[1],
        p;
      !i[0] && d ? (p = ["M"]) : !i[0] && !d && s === 0 ? (p = ["X"]) : (p = [h]);
      for (var v = t.length, _ = 3; _ < v - (v % 3); _ += 3) p.push(u.translate(t.slice(_, _ + 3), _ === v - 3)[0]);
      return (
        (p = p.join("")),
        a.forEach(function (b) {
          if (b[0] % 3 !== 0) throw "Invalid modification coordinate: " + b[0];
          var w = b[0] / 3;
          p = p.slice(0, w) + b[1] + p.slice(w);
        }),
        !i[1] && p[p.length - 1] === "*" && (p = p.slice(0, p.length - 1)),
        p
      );
    }),
    (e.terminalOverlapLen = function (t, s, n, a) {
      ((n = n || Math.min(t.length, s.length)), (a = a || 0));
      var i = n,
        u,
        f,
        h,
        d = function (p, v) {
          for (var _ = 0, b = 0; b < t.length; ++b) p.charAt(b) !== v.charAt(b) && ++_;
          return _;
        };
      for ((t.length < n || s.length < n) && (i = Math.min(t.length, s.length)), u = i; u > 0; ) {
        if (((f = t.slice(-u)), (h = s.slice(0, u)), a === 0)) {
          if (f === h) return u;
        } else if (d(f, h) <= a) return u;
        --u;
      }
      return 0;
    }),
    (e.lcs = function (t, s) {
      if (!t || !s || t.length === 0 || s.length === 0) return 0;
      var n = 0,
        a = 0,
        i = new Array(t.length),
        u,
        f;
      for (u = 0; u <= t.length; u++) for (i[u] = new Array(s.length), f = 0; f <= s.length; f++) i[u][f] = 0;
      for (var h = 0, d = t.length; h < d; h++)
        for (var p = 0, v = s.length; p < v; p++)
          t[h] === s[p]
            ? (h === 0 || p === 0 ? (i[h + 1][p + 1] = 1) : (i[h + 1][p + 1] = i[h][p] + 1),
              i[h + 1][p + 1] > n && ((n = i[h + 1][p + 1]), (a = h)))
            : (i[h + 1][p + 1] = 0);
      return [n, t.slice(a - n + 1, a + 1)];
    }),
    (e.LCS = function (t, s) {
      var n = t.length,
        a = s.length,
        i = [],
        u,
        f;
      for (u = 0; u <= n; u++) i.push([0]);
      for (f = 0; f < a; f++) i[0].push(0);
      for (u = 0; u < n; u++) for (f = 0; f < a; f++) i[u + 1][f + 1] = t[u] === s[f] ? i[u][f] + 1 : Math.max(i[u + 1][f], i[u][f + 1]);
      return (function h(d, p) {
        return d * p === 0
          ? ""
          : t[d - 1] === s[p - 1]
            ? h(d - 1, p - 1) + t[d - 1]
            : i[d][p - 1] > i[d - 1][p]
              ? h(d, p - 1)
              : h(d - 1, p);
      })(n, a);
    }),
    (e.editDist = function (t, s) {
      if (t.length === 0) return s.length;
      if (s.length === 0) return t.length;
      var n = [],
        a,
        i;
      for (a = 0; a <= t.length; a++) n[a] = [a];
      for (i = 0; i <= s.length; i++) n[0][i] = i;
      for (a = 1; a <= t.length; a++)
        for (i = 1; i <= s.length; i++)
          t.charAt(a - 1) === s.charAt(i - 1)
            ? (n[a][i] = n[a - 1][i - 1])
            : (n[a][i] = Math.min(n[a - 1][i - 1] + 1, n[a][i - 1] + 1, n[a - 1][i] + 1));
      return n[t.length][s.length];
    }),
    (e.fGC = function (t) {
      for (var s = 0, n = 0; n < t.length; ++n)
        (t.charAt(n) === "g" || t.charAt(n) === "G" || t.charAt(n) === "C" || t.charAt(n) === "c") && (s += 1);
      var a = s / t.length;
      return a;
    }),
    (e.cntGC = function (t) {
      for (var s = 0, n = 0; n < t.length; ++n)
        (t.charAt(n) === "g" || t.charAt(n) === "G" || t.charAt(n) === "C" || t.charAt(n) === "c") && (s += 1);
      return s;
    }),
    (e.randomDNASeq = function (t, s) {
      if (s && s.a && s.c && s.g && s.t && (s.a + s.c + s.g + s.t > 1 && (s.t = 1 - (s.a + s.c + s.g)), s.t < 0))
        throw new Error("Invalid base distribution");
      for (var n = 0, a, i = ""; n < length; )
        ((a = Math.random()), a < s.a ? (i += "a") : a < s.a + s.c ? (i += "c") : a < s.a + s.c + s.g ? (i += "g") : (i += "t"));
      return i;
    }),
    (e.replicateDNASeq = function (t, s) {
      var n = [];
      for (let a = 0; a < s; ++a) n.push(t);
      return n;
    }),
    (e.cp = function (s) {
      var n = [];
      for (var a of s[0]) n.push([a]);
      for (var i = 1; i < s.length; ) {
        var u = [];
        for (var f of n) for (var a of s[i]) u.push(f.concat([a]));
        ((n = u.slice()), i++);
      }
      return n;
    }),
    (e.expandDNASeq = function (t) {
      const s = /[^acgt]/gi;
      if (!t) return [];
      const n = t.replace(/\s+/g, "");
      for (var a = [], i; (i = s.exec(n)) !== null; ) a.push(i.index);
      var u = [],
        f = 0;
      for (let v of a) (v > f && u.push([n.slice(f, v)]), u.push(e.DEGENERATES[n[v]].split("")), (f = v + 1));
      f < n.length && u.push([n.slice(f)]);
      var h = e.cp(u),
        d = [];
      for (var p of h) d.push(p.join(""));
      return d;
    }),
    (e.NASeq = function (t, s) {
      ((this._modelNASeq = {}),
        (this._modelNASeq.stype = void 0),
        (this._modelNASeq.isDoubleStranded = void 0),
        (this._modelNASeq.topology = "linear"),
        (this._modelNASeq.sid = void 0),
        (this._modelNASeq.sname = void 0),
        (this._modelNASeq.wseq = void 0),
        (this._modelNASeq.cseq = void 0),
        (this._modelNASeq.length = void 0),
        (this._modelNASeq.sym = void 0),
        (this._modelNASeq.gc_cnt = 0),
        (this._modelNASeq.fgc = 0),
        s
          ? (s.stype && this.setSeqType(s.stype),
            s.topology && this.setSeqTopology(s.topology),
            s.sid && this.setSeqId(s.sid),
            s.sname && this.setSeqName(s.sname),
            s.extended ? this.setSeq(t, !0) : this.setSeq(t))
          : this.setSeq(t),
        Object.defineProperty(this, "stype", {
          get: function () {
            return this._modelNASeq.stype;
          },
          set: function (n) {
            this.setSeqType(n);
          },
        }),
        Object.defineProperty(this, "isDoubleStranded", {
          get: function () {
            return this._modelNASeq.isDoubleStranded;
          },
          set: function (n) {
            this._modelNASeq.isDoubleStranded = !!n;
          },
        }),
        Object.defineProperty(this, "sym", {
          get: function () {
            return this._modelNASeq.sym;
          },
          set: function (n) {
            this._modelNASeq.sym = !!n;
          },
        }),
        Object.defineProperty(this, "topology", {
          get: function () {
            return this._modelNASeq.topology;
          },
          set: function (n) {
            this.setSeqTopology(n);
          },
        }),
        Object.defineProperty(this, "sid", {
          get: function () {
            return this._modelNASeq.sid;
          },
          set: function (n) {
            this.setSeqId(n);
          },
        }),
        Object.defineProperty(this, "sname", {
          get: function () {
            return this._modelNASeq.sname;
          },
          set: function (n) {
            this.setSeqName(n);
          },
        }),
        Object.defineProperty(this, "wseq", {
          get: function () {
            return this._modelNASeq.wseq;
          },
          set: function (n) {
            this._modelNASeq.wseq = n;
          },
        }),
        Object.defineProperty(this, "cseq", {
          get: function () {
            return this._modelNASeq.cseq;
          },
          set: function (n) {
            this._modelNASeq.cseq = n;
          },
        }),
        Object.defineProperty(this, "length", {
          get: function () {
            return this._modelNASeq.length;
          },
          set: function (n) {
            this._modelNASeq.length = n;
          },
        }),
        Object.defineProperty(this, "seq", {
          get: function () {
            return this._modelNASeq.wseq;
          },
          set: function (n) {
            this.setSeq(n);
          },
        }));
    }),
    (e.NASeq.prototype = {
      setSeq: function (t) {
        if (((this._modelNASeq.wseq = e.sanitizeSeq(t).toLowerCase()), !e.isValidNASeq(this._modelNASeq.wseq, !1)))
          throw new Error("Invalid seq " + this._modelNASeq.wseq);
        (this._modelNASeq.wseq === "" ? (this._modelNASeq.cseq = "") : (this._modelNASeq.cseq = e.revcomp(this._modelNASeq.wseq)),
          (this._modelNASeq.length = this._modelNASeq.wseq.length),
          (this._modelNASeq.sym = this._modelNASeq.wseq === this._modelNASeq.cseq),
          (this._modelNASeq.gc_cnt = 0));
        for (var s = 0; s < this._modelNASeq.wseq.length; ++s)
          (this._modelNASeq.wseq.charAt(s) === "g" ||
            this._modelNASeq.wseq.charAt(s) === "G" ||
            this._modelNASeq.wseq.charAt(s) === "C" ||
            this._modelNASeq.wseq.charAt(s) === "c") &&
            (this._modelNASeq.gc_cnt += 1);
        return (
          this._modelNASeq.length > 0
            ? (this._modelNASeq.fgc = this._modelNASeq.gc_cnt / this._modelNASeq.wseq.length)
            : (this._modelNASeq.fgc = 0),
          this
        );
      },
      setSeqName: function (t) {
        return ((this._modelNASeq.sname = t), this);
      },
      setSeqId: function (t) {
        return ((this._modelNASeq.sid = t), this);
      },
      setSeqType: function (t) {
        return ((this._modelNASeq.stype = t), this);
      },
      setSeqTopology: function (t) {
        return ((this._modelNASeq.topology = t), this);
      },
      coord: function (t) {
        if (this.topology === "circular" && this.length > 0 && t >= this.length) return t % this.length;
        if (t > this.length) throw new Error("coordinate out of range");
        return t;
      },
      subSeq: function (t, s) {
        if (
          (t < 0 && (t += this._modelNASeq.wseq.length),
          s < 0 && (s += this._modelNASeq.wseq.length),
          t < 0 || t >= this._modelNASeq.wseq.length || s <= 0 || s > this._modelNASeq.wseq.length,
          t < s)
        )
          return this._modelNASeq.wseq.slice(t, s);
        if (this._modelNASeq.topology === "circular") return this._modelNASeq.wseq.slice(t) + this._modelNASeq.wseq.slice(0, s);
        throw new Error("startidx must be < endidx for linear seq");
      },
      splice: function (t, s, n) {
        if (((n = n || ""), arguments.length < 2)) throw new Error("insufficient arguments");
        if ((t < 0 && (t += this._modelNASeq.wseq.length), t < 0 || t >= this._modelNASeq.wseq.length))
          throw new Error("start index out of range");
        return (
          t + s < this.wseq.length
            ? this.setSeq(this._modelNASeq.wseq.slice(0, t) + n + this._modelNASeq.wseq.slice(t + s))
            : this.topology === "linear"
              ? this.setSeq(this._modelNASeq.wseq.slice(0, t) + n)
              : this.setSeq(this._modelNASeq.wseq.slice(t + s - this._modelNASeq.wseq.length, t) + n),
          this
        );
      },
      replace: function (t, s, n) {
        if (((n = n || ""), arguments.length < 2)) throw new Error("insufficient arguments");
        if (t <= s) this.setSeq(this._modelNASeq.wseq.slice(0, t) + n + this._modelNASeq.wseq.slice(s + 1));
        else {
          if (this.topology === "linear") throw new Error("end index smaller than start index");
          this.setSeq(this._modelNASeq.wseq.slice(s + 1, t) + n);
        }
      },
      insert: function (t, s) {
        if (s < 0 || s >= this.wseq.length) throw new Error("start index out of range");
        this.setSeq(this._modelNASeq.wseq.slice(0, s + 1) + t + this._modelNASeq.wseq.slice(s + 1));
      },
      delete: function (t, s) {
        if (t < 0 || t > this._modelNASeq.wseq.length) throw new Error("start index out of range");
        if (s && (s < 0 || s > this._modelNASeq.wseq.length)) throw new Error("end index out of range");
        if (s)
          if (t > s) {
            if (this.topology === "linear") throw new Error("end index smaller than start index");
            this.setSeq(this._modelNASeq.wseq.slice(s, t));
          } else this.setSeq(this._modelNASeq.wseq.slice(0, t) + this._modelNASeq.wseq.slice(s));
        else this.setSeq(this._modelNASeq.wseq.slice(0, t));
      },
      ligate: function (t) {
        return new e.NASeq(this._modelNASeq.wseq + t.wseq);
      },
      shift: function (t) {
        var s;
        t > 0
          ? ((s = this._modelNASeq.wseq.slice(t) + this._modelNASeq.wseq.slice(0, t)), this.setSeq(s))
          : t < 0 &&
            ((s = this._modelNASeq.wseq.slice(this.wseq.length + t) + this._modelNASeq.wseq.slice(0, this._modelNASeq.wseq.length + t)),
            this.setSeq(s));
      },
      toJson: function () {
        return this._modelNASeq;
      },
      expand: function () {
        var t = e.expandDNASeq(this.seq),
          s = this.stype,
          n = this.topology;
        return t.map(function (a) {
          new e.NASeq(a, { stype: s, topology: n });
        });
      },
      deepCopy: function () {
        var t = this._modelNASeq,
          s = new e.NASeq(""),
          n = {};
        for (var a in t) {
          if (t[a] === null) {
            n[a] = null;
            continue;
          }
          if (t[a] === void 0) {
            n[a] = void 0;
            continue;
          }
          n[a] = t[a];
        }
        return ((s._modelNASeq = n), s);
      },
    }),
    (e.parseSeqFile = function (t, s, n) {
      var a = t.trim();
      function i(b) {
        b = b.toLowerCase();
        var w = "",
          y = /\.(\w+)$/.exec(b);
        return (y && y[1] && (w = y[1]), w);
      }
      function u(b) {
        return b.search(/^[\s\r\n]*>/) !== -1 ? h(b) : b.search(/^[\s\r\n]*LOCUS/) !== -1 ? d(b) : f(b);
      }
      function f(b) {
        return { id: "seq", seq: e.sanitizeSeq(b) };
      }
      function h(b) {
        var w, y, A, q, T;
        return (
          (q = b.split(/\r\n|\r|\n/)),
          (A = /^\s*>(\S+)\s*(.*)$/.exec(q[0])),
          (w = A[1] || "seq"),
          w[w.length - 1] === " " && (w = w.splice(0, w.length - 1)),
          (q = q.slice(1)),
          (y = q.join("")),
          (T = { id: w, seq: e.sanitizeSeq(y), isCircular: !1 }),
          T
        );
      }
      function d(b) {
        var w = b.split(/\r\n|\r|\n/),
          y = /^LOCUS\s+(\w+)\s/,
          A = y.exec(w[0]),
          q = (A && A[1]) || "seq",
          T = !1;
        w[0].indexOf("circular") !== -1 && (T = !0);
        for (var N = 0; !w[N++].startsWith("ORIGIN"); );
        w.splice(0, N);
        var M = w.join(""),
          k = { id: q, seq: e.sanitizeSeq(M), isCircular: T };
        return k;
      }
      var p = { any: u, raw: f, fasta: h, genbank: d };
      ((n = n || "any"), (n = n.toLowerCase()));
      var v;
      if (!n && s) {
        var _ = i(s);
        switch (_) {
          case "fa":
          case "fasta":
          case "fas":
          case "fna":
          case "ffn":
            n = "fasta";
            break;
          case "gb":
          case "gbk":
            n = "genbank";
            break;
          default:
            n = "any";
        }
      } else n = "any";
      if (n in p) v = p[n](a);
      else throw new Error("parseSeqFile: invalid sequence format type");
      return v;
    }),
    e
  );
})(ao);
const Te = ao;
var co = {};
(function (l) {
  var e = l;
  return (
    (e.TmCalc = function (t) {
      this.init(t);
    }),
    (e.TmCalc.prototype.nnBr = {
      aa: { dh: -9.1, ds: -22.2 },
      tt: { dh: -9.1, ds: -22.2 },
      at: { dh: -8.6, ds: -20.4 },
      ta: { dh: -6, ds: -21.3 },
      ca: { dh: -5.8, ds: -22.7 },
      tg: { dh: -5.8, ds: -22.7 },
      gt: { dh: -6.5, ds: -22.4 },
      ac: { dh: -6.5, ds: -22.4 },
      ct: { dh: -7.8, ds: -21 },
      ag: { dh: -7.8, ds: -21 },
      ga: { dh: -5.6, ds: -22.2 },
      tc: { dh: -5.6, ds: -22.2 },
      cg: { dh: -11.9, ds: -27.2 },
      gc: { dh: -11.1, ds: -24.4 },
      gg: { dh: -11, ds: -19.9 },
      cc: { dh: -11, ds: -19.9 },
    }),
    (e.TmCalc.prototype.nnSa = {
      aa: { dh: -7.9, ds: -22.2 },
      tt: { dh: -7.9, ds: -22.2 },
      at: { dh: -7.2, ds: -20.4 },
      ta: { dh: -7.2, ds: -21.3 },
      ca: { dh: -8.5, ds: -22.7 },
      tg: { dh: -8.5, ds: -22.7 },
      gt: { dh: -8.4, ds: -22.4 },
      ac: { dh: -8.4, ds: -22.4 },
      ct: { dh: -7.8, ds: -21 },
      ag: { dh: -7.8, ds: -21 },
      ga: { dh: -8.2, ds: -22.2 },
      tc: { dh: -8.2, ds: -22.2 },
      cg: { dh: -10.6, ds: -27.2 },
      gc: { dh: -9.8, ds: -24.4 },
      gg: { dh: -8, ds: -19.9 },
      cc: { dh: -8, ds: -19.9 },
    }),
    (e.TmCalc.prototype.nnde = {
      "-a/at": { dh: -0.7, ds: -0.8 },
      "-a/ct": { dh: 4.4, ds: 14.9 },
      "-a/gt": { dh: -1.6, ds: -3.6 },
      "-a/tt": { dh: 2.9, ds: 10.4 },
      "-c/ag": { dh: -2.1, ds: -3.9 },
      "-c/cg": { dh: -0.2, ds: -0.1 },
      "-c/gg": { dh: -3.9, ds: -11.2 },
      "-c/tg": { dh: -4.4, ds: -13.1 },
      "-g/ac": { dh: -5.9, ds: -16.5 },
      "-g/cc": { dh: -2.6, ds: -7.4 },
      "-g/gc": { dh: -3.2, ds: -10.4 },
      "-g/tc": { dh: -5.2, ds: -15 },
      "-t/aa": { dh: -0.5, ds: -1.1 },
      "-t/ca": { dh: 4.7, ds: 7.7 },
      "-t/ga": { dh: -4.1, ds: -13.1 },
      "-t/ta": { dh: -3.8, ds: -12.6 },
      "a-/ta": { dh: -2.9, ds: -7.6 },
      "a-/tc": { dh: -4.1, ds: -13 },
      "a-/tg": { dh: -4.2, ds: -15 },
      "a-/tt": { dh: -0.2, ds: -0.5 },
      "aa/-t": { dh: 0.2, ds: 2.3 },
      "aa/t-": { dh: -0.5, ds: -1.1 },
      "ac/-g": { dh: -6.3, ds: -17.1 },
      "ac/t-": { dh: 4.7, ds: 7.7 },
      "ag/-c": { dh: -3.7, ds: -10 },
      "ag/t-": { dh: -4.1, ds: -13.1 },
      "at/-a": { dh: -2.9, ds: -7.6 },
      "at/t-": { dh: -3.8, ds: -12.6 },
      "c-/ga": { dh: -3.7, ds: -10 },
      "c-/gc": { dh: -4, ds: -11.9 },
      "c-/gg": { dh: -3.9, ds: -10.9 },
      "c-/gt": { dh: -4.9, ds: -13.8 },
      "ca/-t": { dh: 0.6, ds: 3.3 },
      "ca/g-": { dh: -5.9, ds: -16.5 },
      "cc/-g": { dh: -4.4, ds: -12.6 },
      "cc/g-": { dh: -2.6, ds: -7.4 },
      "cg/-c": { dh: -4, ds: -11.9 },
      "cg/g-": { dh: -3.2, ds: -10.4 },
      "ct/-a": { dh: -4.1, ds: -13 },
      "ct/g-": { dh: -5.2, ds: -15 },
      "g-/ca": { dh: -6.3, ds: -17.1 },
      "g-/cc": { dh: -4.4, ds: -12.6 },
      "g-/cg": { dh: -5.1, ds: -14 },
      "g-/ct": { dh: -4, ds: -10.9 },
      "ga/-t": { dh: -1.1, ds: -1.6 },
      "ga/c-": { dh: -2.1, ds: -3.9 },
      "gc/-g": { dh: -5.1, ds: -14 },
      "gc/c-": { dh: -0.2, ds: -0.1 },
      "gg/-c": { dh: -3.9, ds: -10.9 },
      "gg/c-": { dh: -3.9, ds: -11.2 },
      "gt/-a": { dh: -4.2, ds: -15 },
      "gt/c-": { dh: -4.4, ds: -13.1 },
      "t-/aa": { dh: 0.2, ds: 2.3 },
      "t-/ac": { dh: 0.6, ds: 3.3 },
      "t-/ag": { dh: -1.1, ds: -1.6 },
      "t-/at": { dh: -6.9, ds: -20 },
      "ta/-t": { dh: -6.9, ds: -20 },
      "ta/a-": { dh: -0.7, ds: -0.8 },
      "tc/-g": { dh: -4, ds: -10.9 },
      "tc/a-": { dh: 4.4, ds: 14.9 },
      "tg/-c": { dh: -4.9, ds: -13.8 },
      "tg/a-": { dh: -1.6, ds: -3.6 },
      "tt/-a": { dh: -0.2, ds: -0.5 },
      "tt/a-": { dh: 2.9, ds: 10.4 },
    }),
    (e.TmCalc.prototype.nndhds = {
      "aa/at": { dh: 4.7, ds: 12.9 },
      "aa/ct": { dh: 7.6, ds: 20.2 },
      "aa/gt": { dh: 3, ds: 7.4 },
      "aa/ta": { dh: 1.2, ds: 1.7 },
      "aa/tc": { dh: 2.3, ds: 4.6 },
      "aa/tg": { dh: -0.6, ds: -2.3 },
      "aa/tt": { dh: -7.6, ds: -21.3 },
      "ac/ag": { dh: -2.9, ds: -9.8 },
      "ac/cg": { dh: -0.7, ds: -3.8 },
      "ac/gg": { dh: 0.5, ds: 3.2 },
      "ac/ta": { dh: 5.3, ds: 14.6 },
      "ac/tc": { dh: 0, ds: -4.4 },
      "ac/tg": { dh: -8.4, ds: -22.4 },
      "ac/tt": { dh: 0.7, ds: 0.2 },
      "ag/ac": { dh: -0.9, ds: -4.2 },
      "ag/cc": { dh: 0.6, ds: -0.6 },
      "ag/gc": { dh: -4, ds: -13.2 },
      "ag/ta": { dh: -0.7, ds: -2.3 },
      "ag/tc": { dh: -7.8, ds: -21 },
      "ag/tg": { dh: -3.1, ds: -9.5 },
      "ag/tt": { dh: 1, ds: 0.9 },
      "at/aa": { dh: 1.2, ds: 1.7 },
      "at/ca": { dh: 5.3, ds: 14.6 },
      "at/ga": { dh: -0.7, ds: -2.3 },
      "at/ta": { dh: -7.2, ds: -20.4 },
      "at/tc": { dh: -1.2, ds: -6.2 },
      "at/tg": { dh: -2.5, ds: -8.3 },
      "at/tt": { dh: -2.7, ds: -10.8 },
      "ca/at": { dh: 3.4, ds: 8 },
      "ca/ct": { dh: 6.1, ds: 16.4 },
      "ca/ga": { dh: -0.9, ds: -4.2 },
      "ca/gc": { dh: 1.9, ds: 3.7 },
      "ca/gg": { dh: -0.7, ds: -2.3 },
      "ca/gt": { dh: -8.5, ds: -22.7 },
      "ca/tt": { dh: 1, ds: 0.7 },
      "cc/ag": { dh: 5.2, ds: 14.2 },
      "cc/cg": { dh: 3.6, ds: 8.9 },
      "cc/ga": { dh: 0.6, ds: -0.6 },
      "cc/gc": { dh: -1.5, ds: -7.2 },
      "cc/gg": { dh: -8, ds: -19.9 },
      "cc/gt": { dh: -0.8, ds: -4.5 },
      "cc/tg": { dh: 5.2, ds: 13.5 },
      "cg/ac": { dh: 1.9, ds: 3.7 },
      "cg/cc": { dh: -1.5, ds: -7.2 },
      "cg/ga": { dh: -4, ds: -13.2 },
      "cg/gc": { dh: -10.6, ds: -27.2 },
      "cg/gg": { dh: -4.9, ds: -15.3 },
      "cg/gt": { dh: -4.1, ds: -11.7 },
      "cg/tc": { dh: -1.5, ds: -6.1 },
      "ct/aa": { dh: 2.3, ds: 4.6 },
      "ct/ca": { dh: 0, ds: -4.4 },
      "ct/ga": { dh: -7.8, ds: -21 },
      "ct/gc": { dh: -1.5, ds: -6.1 },
      "ct/gg": { dh: -2.8, ds: -8 },
      "ct/gt": { dh: -5, ds: -15.8 },
      "ct/ta": { dh: -1.2, ds: -6.2 },
      "ga/at": { dh: 0.7, ds: 0.7 },
      "ga/ca": { dh: -2.9, ds: -9.8 },
      "ga/cc": { dh: 5.2, ds: 14.2 },
      "ga/cg": { dh: -0.6, ds: -1 },
      "ga/ct": { dh: -8.2, ds: -22.2 },
      "ga/gt": { dh: 1.6, ds: 3.6 },
      "ga/tt": { dh: -1.3, ds: -5.3 },
      "gc/ag": { dh: -0.6, ds: -1 },
      "gc/ca": { dh: -0.7, ds: -3.8 },
      "gc/cc": { dh: 3.6, ds: 8.9 },
      "gc/cg": { dh: -9.8, ds: -24.4 },
      "gc/ct": { dh: 2.3, ds: 5.4 },
      "gc/gg": { dh: -6, ds: -15.8 },
      "gc/tg": { dh: -4.4, ds: -12.3 },
      "gg/ac": { dh: -0.7, ds: -2.3 },
      "gg/ca": { dh: 0.5, ds: 3.2 },
      "gg/cc": { dh: -8, ds: -19.9 },
      "gg/cg": { dh: -6, ds: -15.8 },
      "gg/ct": { dh: 3.3, ds: 10.4 },
      "gg/gc": { dh: -4.9, ds: -15.3 },
      "gg/tc": { dh: -2.8, ds: -8 },
      "gt/aa": { dh: -0.6, ds: -2.3 },
      "gt/ca": { dh: -8.4, ds: -22.4 },
      "gt/cc": { dh: 5.2, ds: 13.5 },
      "gt/cg": { dh: -4.4, ds: -12.3 },
      "gt/ct": { dh: -2.2, ds: -8.4 },
      "gt/ga": { dh: -3.1, ds: -9.5 },
      "gt/ta": { dh: -2.5, ds: -8.3 },
      "ta/aa": { dh: 4.7, ds: 12.9 },
      "ta/ac": { dh: 3.4, ds: 8 },
      "ta/ag": { dh: 0.7, ds: 0.7 },
      "ta/at": { dh: -7.2, ds: -21.3 },
      "ta/ct": { dh: 1.2, ds: 0.7 },
      "ta/gt": { dh: -0.1, ds: -1.7 },
      "ta/tt": { dh: 0.2, ds: -1.5 },
      "tc/aa": { dh: 7.6, ds: 20.2 },
      "tc/ac": { dh: 6.1, ds: 16.4 },
      "tc/ag": { dh: -8.2, ds: -22.2 },
      "tc/at": { dh: 1.2, ds: 0.7 },
      "tc/cg": { dh: 2.3, ds: 5.4 },
      "tc/gg": { dh: 3.3, ds: 10.4 },
      "tc/tg": { dh: -2.2, ds: -8.4 },
      "tg/aa": { dh: 3, ds: 7.4 },
      "tg/ac": { dh: -8.5, ds: -22.7 },
      "tg/ag": { dh: 1.6, ds: 3.6 },
      "tg/at": { dh: -0.1, ds: -1.7 },
      "tg/cc": { dh: -0.8, ds: -4.5 },
      "tg/gc": { dh: -4.1, ds: -11.7 },
      "tg/tc": { dh: -5, ds: -15.8 },
      "tt/aa": { dh: -7.6, ds: -21.3 },
      "tt/ac": { dh: 1, ds: 0.7 },
      "tt/ag": { dh: -1.3, ds: -5.3 },
      "tt/at": { dh: 0.2, ds: -1.5 },
      "tt/ca": { dh: 0.7, ds: 0.2 },
      "tt/ga": { dh: 1, ds: 0.9 },
      "tt/ta": { dh: -2.7, ds: -10.8 },
      "ai/tc": { dh: -8.9, ds: -25.5 },
      "ti/ac": { dh: -5.9, ds: -17.4 },
      "ac/ti": { dh: -8.8, ds: -25.4 },
      "tc/ai": { dh: -4.9, ds: -13.9 },
      "ci/gc": { dh: -5.4, ds: -13.7 },
      "gi/cc": { dh: -6.8, ds: -19.1 },
      "cc/gi": { dh: -8.3, ds: -23.8 },
      "gc/ci": { dh: -5, ds: -12.6 },
      "ai/ta": { dh: -8.3, ds: -25 },
      "ti/aa": { dh: -3.4, ds: -11.2 },
      "aa/ti": { dh: -0.7, ds: -2.6 },
      "ta/ai": { dh: -1.3, ds: -4.6 },
      "ci/ga": { dh: 2.6, ds: 8.9 },
      "gi/ca": { dh: -7.8, ds: -21.1 },
      "ca/gi": { dh: -7, ds: -20 },
      "ga/ci": { dh: -7.6, ds: -20.2 },
      "ai/tt": { dh: 0.49, ds: -0.7 },
      "ti/at": { dh: -6.5, ds: -22 },
      "at/ti": { dh: -5.6, ds: -18.7 },
      "tt/ai": { dh: -0.8, ds: -4.3 },
      "ci/gt": { dh: -1, ds: -2.4 },
      "gi/ct": { dh: -3.5, ds: -10.6 },
      "ct/gi": { dh: 0.1, ds: -1 },
      "gt/ci": { dh: -4.3, ds: -12.1 },
      "ai/tg": { dh: -4.9, ds: -15.8 },
      "ti/ag": { dh: -1.9, ds: -8.5 },
      "ag/ti": { dh: 0.1, ds: -1.8 },
      "tg/ai": { dh: 1, ds: 1 },
      "ci/gg": { dh: 7.1, ds: 21.3 },
      "gi/cg": { dh: -1.1, ds: -3.2 },
      "cg/gi": { dh: 5.8, ds: 16.9 },
      "gg/ci": { dh: -7.6, ds: -22 },
      "ai/ti": { dh: -3.3, ds: -11.9 },
      "ti/ai": { dh: 0.1, ds: -2.3 },
      "ci/gi": { dh: 1.3, ds: 3 },
      "gi/ci": { dh: -0.5, ds: -1.3 },
    }),
    (e.TmCalc.prototype.nnloop = {
      3: { dh: 0, ds: -10.3 },
      4: { dh: 0, ds: -11.6 },
      5: { dh: 0, ds: -12.9 },
      6: { dh: 0, ds: -14.2 },
      7: { dh: 0, ds: -14.8 },
      8: { dh: 0, ds: -15.5 },
      9: { dh: 0, ds: -15.8 },
      10: { dh: 0, ds: -15.8 },
      12: { dh: 0, ds: -16.8 },
      14: { dh: 0, ds: -17.4 },
      16: { dh: 0, ds: -18.1 },
      18: { dh: 0, ds: -18.7 },
      20: { dh: 0, ds: -19 },
      25: { dh: 0, ds: -20.3 },
      30: { dh: 0, ds: -21.3 },
    }),
    (e.TmCalc.prototype.nnbulge = {
      1: { dh: 0, ds: -12.9 },
      2: { dh: 0, ds: -9.4 },
      3: { dh: 0, ds: -10 },
      4: { dh: 0, ds: -10.3 },
      5: { dh: 0, ds: -10.6 },
      6: { dh: 0, ds: -11.3 },
      7: { dh: 0, ds: -11.9 },
      8: { dh: 0, ds: -12.6 },
      9: { dh: 0, ds: -13.2 },
      10: { dh: 0, ds: -13.9 },
      12: { dh: 0, ds: -14.5 },
      14: { dh: 0, ds: -15.5 },
      16: { dh: 0, ds: -16.1 },
      18: { dh: 0, ds: -16.8 },
      20: { dh: 0, ds: -17.1 },
      25: { dh: 0, ds: -18.1 },
      30: { dh: 0, ds: -19 },
    }),
    (e.TmCalc.prototype.nntmm = {
      "aa/ta": { dh: -3.1, ds: -7.8 },
      "aa/tc": { dh: -1.6, ds: -4 },
      "aa/tg": { dh: -1.9, ds: -4.4 },
      "ac/ta": { dh: -1.8, ds: -3.8 },
      "ac/tc": { dh: -0.1, ds: 0.5 },
      "ac/tt": { dh: -0.9, ds: -1.7 },
      "ag/ta": { dh: -2.5, ds: -5.9 },
      "ag/tg": { dh: -1.1, ds: -2.1 },
      "ag/tt": { dh: -3.2, ds: -8.7 },
      "at/tc": { dh: -2.3, ds: -6.3 },
      "at/tg": { dh: -3.5, ds: -9.4 },
      "at/tt": { dh: -2.4, ds: -6.5 },
      "ca/ga": { dh: -4.3, ds: -10.7 },
      "ca/gc": { dh: -2.6, ds: -5.9 },
      "ca/gg": { dh: -3.9, ds: -9.6 },
      "cc/ga": { dh: -2.7, ds: -6 },
      "cc/gc": { dh: -2.1, ds: -5.1 },
      "cc/gt": { dh: -3.2, ds: -8 },
      "cg/ga": { dh: -6, ds: -15.5 },
      "cg/gg": { dh: -3.8, ds: -9.5 },
      "cg/gt": { dh: -3.8, ds: -9 },
      "ct/gc": { dh: -3.9, ds: -10.6 },
      "ct/gg": { dh: -6.6, ds: -18.7 },
      "ct/gt": { dh: -6.1, ds: -16.9 },
      "ga/ca": { dh: -8, ds: -22.5 },
      "ga/cc": { dh: -5, ds: -13.8 },
      "ga/cg": { dh: -4.3, ds: -11.1 },
      "gc/ca": { dh: -3.2, ds: -7.1 },
      "gc/cc": { dh: -3.9, ds: -10.6 },
      "gc/ct": { dh: -4.9, ds: -13.5 },
      "gg/ca": { dh: -4.6, ds: -11.4 },
      "gg/cg": { dh: -0.7, ds: -19.2 },
      "gg/ct": { dh: -5.7, ds: -15.9 },
      "gt/cc": { dh: -3, ds: -7.8 },
      "gt/cg": { dh: -5.9, ds: -16.1 },
      "gt/ct": { dh: -7.4, ds: -21.2 },
      "ta/aa": { dh: -2.5, ds: -6.3 },
      "ta/ac": { dh: -2.3, ds: -5.9 },
      "ta/ag": { dh: -2, ds: -4.7 },
      "tc/aa": { dh: -2.7, ds: -7 },
      "tc/ac": { dh: -0.7, ds: -1.3 },
      "tc/at": { dh: -2.5, ds: -6.3 },
      "tg/aa": { dh: -2.4, ds: -5.8 },
      "tg/ag": { dh: -1.1, ds: -2.7 },
      "tg/at": { dh: -3.9, ds: -10.5 },
      "tt/ac": { dh: -0.7, ds: -1.2 },
      "tt/ag": { dh: -3.6, ds: -9.8 },
      "tt/at": { dh: -3.2, ds: -8.9 },
      "at/aa": { dh: -3.1, ds: -7.8 },
      "ct/aa": { dh: -1.6, ds: -4 },
      "gt/aa": { dh: -1.9, ds: -4.4 },
      "at/ca": { dh: -1.8, ds: -3.8 },
      "ct/ca": { dh: -0.1, ds: 0.5 },
      "tt/ca": { dh: -0.9, ds: -1.7 },
      "at/ga": { dh: -2.5, ds: -5.9 },
      "gt/ga": { dh: -1.1, ds: -2.1 },
      "tt/ga": { dh: -3.2, ds: -8.7 },
      "ct/ta": { dh: -2.3, ds: -6.3 },
      "gt/ta": { dh: -3.5, ds: -9.4 },
      "tt/ta": { dh: -2.4, ds: -6.5 },
      "ag/ac": { dh: -4.3, ds: -10.7 },
      "cg/ac": { dh: -2.6, ds: -5.9 },
      "gg/ac": { dh: -3.9, ds: -9.6 },
      "ag/cc": { dh: -2.7, ds: -6 },
      "cg/cc": { dh: -2.1, ds: -5.1 },
      "tg/cc": { dh: -3.2, ds: -8 },
      "ag/gc": { dh: -6, ds: -15.5 },
      "gg/gc": { dh: -3.8, ds: -9.5 },
      "tg/gc": { dh: -3.8, ds: -9 },
      "cg/tc": { dh: -3.9, ds: -10.6 },
      "gg/tc": { dh: -6.6, ds: -18.7 },
      "tg/tc": { dh: -6.1, ds: -16.9 },
      "ac/ag": { dh: -8, ds: -22.5 },
      "cc/ag": { dh: -5, ds: -13.8 },
      "gc/ag": { dh: -4.3, ds: -11.1 },
      "ac/cg": { dh: -3.2, ds: -7.1 },
      "cc/cg": { dh: -3.9, ds: -10.6 },
      "tc/cg": { dh: -4.9, ds: -13.5 },
      "ac/gg": { dh: -4.6, ds: -11.4 },
      "gc/gg": { dh: -0.7, ds: -19.2 },
      "tc/gg": { dh: -5.7, ds: -15.9 },
      "cc/tg": { dh: -3, ds: -7.8 },
      "gc/tg": { dh: -5.9, ds: -16.1 },
      "tc/tg": { dh: -7.4, ds: -21.2 },
      "aa/at": { dh: -2.5, ds: -6.3 },
      "ca/at": { dh: -2.3, ds: -5.9 },
      "ga/at": { dh: -2, ds: -4.7 },
      "aa/ct": { dh: -2.7, ds: -7 },
      "ca/ct": { dh: -0.7, ds: -1.3 },
      "ta/ct": { dh: -2.5, ds: -6.3 },
      "aa/gt": { dh: -2.4, ds: -5.8 },
      "ga/gt": { dh: -1.1, ds: -2.7 },
      "ta/gt": { dh: -3.9, ds: -10.5 },
      "ca/tt": { dh: -0.7, ds: -1.2 },
      "ga/tt": { dh: -3.6, ds: -9.8 },
      "ta/tt": { dh: -3.2, ds: -8.9 },
    }),
    (e.TmCalc.prototype.R = 1.987),
    (e.TmCalc.prototype.dSBr = {
      aa: -24,
      tt: -24,
      at: -23.9,
      ta: -16.9,
      ca: -12.9,
      tg: -12.9,
      gt: -17.3,
      ac: -17.3,
      ct: -20.8,
      ag: -20.8,
      ga: -13.5,
      tc: -13.5,
      cg: -27.8,
      gc: -26.7,
      gg: -26.6,
      cc: -26.6,
    }),
    (e.TmCalc.prototype.dSSa = {
      aa: -22.2,
      tt: -22.2,
      at: -20.4,
      ta: -21.3,
      ca: -22.7,
      tg: -22.7,
      gt: -22.4,
      ac: -22.4,
      ct: -21,
      ag: -21,
      ga: -22.2,
      tc: -22.2,
      cg: -27.2,
      gc: -24.4,
      gg: -19.9,
      cc: -19.9,
    }),
    (e.TmCalc.prototype.dHBr = {
      aa: -9.1,
      tt: -9.1,
      at: -8.6,
      ta: -6,
      ca: -5.8,
      tg: -5.8,
      gt: -6.5,
      ac: -6.5,
      ct: -7.8,
      ag: -7.8,
      ga: -5.6,
      tc: -5.6,
      cg: -11.9,
      gc: -11.1,
      gg: -11,
      cc: -11,
    }),
    (e.TmCalc.prototype.dHSa = {
      aa: -7.9,
      tt: -7.9,
      at: -7.2,
      ta: -7.2,
      ca: -8.5,
      tg: -8.5,
      gt: -8.4,
      ac: -8.4,
      ct: -7.8,
      ag: -7.8,
      ga: -8.2,
      tc: -8.2,
      cg: -10.6,
      gc: -9.8,
      gg: -8,
      cc: -8,
    }),
    (e.TmCalc.prototype.init = function (t) {
      (t
        ? t.seq && t.compseq
          ? this.setSeq(t.seq, t.compseq)
          : t.seq
            ? this.setSeq(t.seq)
            : ((this.wseq = ""), (this.cseq = ""))
        : ((this.wseq = ""), (this.cseq = ""), (t = {})),
        this.setCt(t.ct || 0.25),
        this.setMonosalt(t.monosalt || 50),
        this.setDisalt(t.disalt || 0),
        this.setDMSO(t.dmso || 0),
        this.setMethod(t.method || 4),
        (this.pairmap = t.pairmap || ""));
    }),
    (e.TmCalc.prototype.saltCorrect = function () {
      var t, s, n, a, i, u, f, h;
      if (this.saltc === 0 && this.disaltc === 0)
        throw { name: "Missing salt", message: "Unable to calculate salt correction because mono and divalent salt are missing." };
      ((this.sc_sch = (16.6 * Math.log(this.saltc)) / Math.LN10),
        (this.sc_sl = 0.368 * this.wseq.length * Math.log(this.saltc)),
        (n = -0.911),
        (a = 6.26),
        (u = -48.2),
        (f = 52.5),
        (s = 3.92),
        (i = 1.42),
        (h = 8.31),
        this.saltc > 0 ? (t = Math.sqrt(this.disaltc) / this.saltc) : (t = -1),
        t >= 0 && t < 0.22
          ? (this.sc_ow = (4.29 * this.fgc - 3.95) * 1e-5 * Math.log(this.saltc) + 94e-7 * Math.log(this.saltc) * Math.log(this.saltc))
          : (t >= 0.22 &&
              t < 6 &&
              ((s = 3.92 * (0.843 - 0.352 * Math.sqrt(this.saltc) * Math.log(this.saltc))),
              (i = 1.42 * (1.279 - 0.00403 * Math.log(this.saltc) - 0.00803 * Math.log(this.saltc) * Math.log(this.saltc))),
              (h =
                8.31 *
                (0.486 - 0.258 * Math.log(this.saltc) + 0.00525 * Math.log(this.saltc) * Math.log(this.saltc) * Math.log(this.saltc)))),
            (this.sc_ow =
              (s +
                n * Math.log(this.disaltc) +
                this.fgc * (a + i * Math.log(this.disaltc)) +
                (0.5 / (this.wseq.length - 1)) * (u + f * Math.log(this.disaltc) + h * Math.log(this.disaltc) * Math.log(this.disaltc))) *
              1e-5)));
    }),
    (e.TmCalc.prototype.setMethod = function (t) {
      return ((this.method = t), this);
    }),
    (e.TmCalc.prototype.setSeq = function (t, s) {
      var n;
      for (
        this.wseq = t.toLowerCase().replace(/\s/g, "").replace(/u/g, "t"),
          s
            ? (this.cseq = s.toLowerCase().replace(/\s/g, ""))
            : ((this.cseq = this.revcomp(this.wseq, !1)), (this.pairmap = Array(this.wseq.length + 1).join("|"))),
          this.sym = this.wseq === this.cseq,
          this.gc_cnt = 0,
          this.at_cnt = 0,
          n = 0;
        n < this.wseq.length;
        ++n
      )
        this.wseq.charAt(n) === "g" || this.wseq.charAt(n) === "c"
          ? (this.gc_cnt += 1)
          : (this.wseq.charAt(n) === "a" || this.wseq.charAt(n) === "t") && (this.at_cnt += 1);
      return ((this.fgc = this.gc_cnt / (this.at_cnt + this.gc_cnt)), this.saltCorrect(), this);
    }),
    (e.TmCalc.prototype.setCt = function (t) {
      return ((this.ct = t), (this.primerc = this.ct * 1e-6), this);
    }),
    (e.TmCalc.prototype.setMonosalt = function (t) {
      return ((this.salt = t), (this.saltc = this.salt * 0.001), this.saltCorrect(), this);
    }),
    (e.TmCalc.prototype.setDisalt = function (t) {
      return ((this.disalt = t), (this.disaltc = this.disalt * 0.001), this.saltCorrect(), this);
    }),
    (e.TmCalc.prototype.setDMSO = function (t) {
      return ((this.dmso = t), this);
    }),
    (e.TmCalc.prototype.revcomp = function t(s) {
      if (
        (t.COMPLEMENTS ||
          (t.COMPLEMENTS = (function () {
            var f,
              h = "ACGTUNSWMKRYVDHBacgtunswmkryvdhb -",
              d = "TGCAANSWKMYRBHDVtgcaanswkmyrbhdv -",
              p = {};
            for (f = 0; f < h.length; ++f) p[h.charAt(f)] = d.charAt(f);
            return p;
          })()),
        (s = s || ""),
        s === "")
      )
        return "";
      for (var n = s.split(""), a = 0; a < n.length; ++a) {
        if (!t.COMPLEMENTS[n[a]]) throw "Unknown base: " + n[a];
        n[a] = t.COMPLEMENTS[n[a]];
      }
      for (var i = "", u = n.length - 1; u >= 0; u--) i += n[u];
      return i;
    }),
    (e.TmCalc.prototype.comp = function (s) {
      if (
        (this.COMPLEMENTS ||
          (this.COMPLEMENTS = (function () {
            var u,
              f = "ACGTUNSWMKRYVDHBacgtunswmkryvdhb -",
              h = "TGCAANSWKMYRBHDVtgcaanswkmyrbhdv -",
              d = {};
            for (u = 0; u < f.length; ++u) d[f.charAt(u)] = h.charAt(u);
            return d;
          })()),
        (s = s || ""),
        s === "")
      )
        return "";
      for (var n = s.split(""), a = 0; a < n.length; ++a) {
        if (!this.COMPLEMENTS[n[a]]) throw "Unknown base: " + n[a];
        n[a] = this.COMPLEMENTS[n[a]];
      }
      var i = n.join("");
      return i;
    }),
    (e.TmCalc.prototype.calcTm_nomm = function (t) {
      if ((!t || t === "") && !this.wseq) return {};
      t && (this.setSeq(t), this.saltCorrect());
      var s = 0,
        n = 0,
        a = 0,
        i = 0,
        u = 0,
        f = 0,
        h = this.ct * 1e-6,
        d,
        p,
        v = 0;
      switch (this.method) {
        case 1:
          for (i += this.sym ? -12.4 : -10.8, d = 0; d < this.wseq.length - 1; ++d)
            ((p = this.wseq.slice(d, d + 2)), (n += this.dSBr[p]), (a += this.dHBr[p]));
          ((a *= 1e3), (v = a / (i + n + this.R * Math.log(h)) - 273.15), (s = v + this.sc_sch));
          break;
        case 3:
        case 4:
        case 5:
          for (
            f = this.sym ? -1.4 : 0,
              (this.wseq.charAt(0) === "a" || this.wseq.charAt(0) === "t") && ((i += 4.1), (u += 2300)),
              (this.wseq.charAt(this.wseq.length - 1) === "a" || this.wseq.charAt(this.wseq.length - 1) === "t") &&
                ((i += 4.1), (u += 2300)),
              (this.wseq.charAt(0) === "g" || this.wseq.charAt(0) === "c") && ((i += -2.8), (u += 100)),
              (this.wseq.charAt(this.wseq.length - 1) === "g" || this.wseq.charAt(this.wseq.length - 1) === "c") &&
                ((i += -2.8), (u += 100)),
              d = 0;
            d < this.wseq.length - 1;
            d++
          )
            ((p = this.wseq.slice(d, d + 2)), (n += this.dSSa[p]), (a += this.dHSa[p]));
          ((a *= 1e3),
            (s = (a + u) / (i + f + n + this.R * Math.log(h))),
            (v = s - 273.15),
            this.method === 3
              ? (s = 1 / (1 / s + this.sc_sl / (a + u)))
              : this.method === 4
                ? (s = 1 / (1 / s + this.sc_ow))
                : this.method === 5 && (s = s + this.sc_sch),
            (s -= 273.15));
          break;
        case 7:
          s = 81.5 + (16.6 * Math.log(this.saltc)) / Math.LN10 + 0.41 * this.fgc - 675 / this.wseq.length;
          break;
      }
      return (
        (s = s - this.dmso * 0.6),
        {
          method: this.method,
          wseq: this.wseq,
          rawtm: v,
          tm: s,
          dh: a + u,
          ds: n + i + f,
          salt: this.saltc,
          ct: this.primerc,
          dmso: this.dmso,
          fgc: this.fgc,
          len: this.wseq.length,
        }
      );
    }),
    e
  );
})(co);
const Yo = co;
var uo = {};
(function (l) {
  var e = l,
    t = Te,
    s = Yo;
  return (
    (e.simpleTm = function (n) {
      for (var a = n.toLowerCase(), i = a.length, u = 0, f = 0; f < a.length; ++f) (a.charAt(f) === "g" || a.charAt(f) === "c") && (u += 1);
      return 4 * u + 2 * (i - u);
    }),
    (e.createTmFunction = function (n, a, i) {
      var u = i || 250,
        f = a || 50,
        h = n || 4;
      ((u = u / 1e3), (h === 1 || h === 5) && (u = u / 4));
      var d = new s.TmCalc({ ct: u, monosalt: f, method: h }),
        p = function (v, _) {
          return d.calcTm_nomm(v).tm;
        };
      return p;
    }),
    (e.Primer = function (n, a, i) {
      var u, f, h, d;
      if (
        ((i = i || { primerMethod: 4, primerMonosalt: 50, primerCt: 200, primerMinLen: 20, primerMinTm: 55, primerMaxTm: 70 }),
        i &&
          i.primerMonosalt &&
          i.primerCt &&
          ((d = i.primerMonosalt), (h = i.primerCt), i.primerMethod ? (f = i.primerMethod) : (f = 4), (u = e.createTmFunction(f, d, h))),
        i && i.primerMinLen && (this.primerMinLen = i.primerMinLen),
        i && i.primerMinTm && (this.primerMinTm = i.primerMinTm),
        i && i.primerMaxTm && (this.primerMaxTm = i.maxTm),
        (a = a || ""),
        (this._modelPrimer = {}),
        (this._modelPrimer.tm = null),
        (this._modelPrimer.ta = null),
        (this._modelPrimer.primerMonosalt = i.primerMonosalt || 150),
        (this._modelPrimer.primerDisalt = i.primerDisalt || 0),
        (this._modelPrimer.primerCt = i.primerCt || 500),
        (this._modelPrimer.primerMethod = i.primerMethod || 4),
        (this._modelPrimer.primerMinLen = i.primerMinLen || 20),
        (this._modelPrimer.primerMinTm = i.primerMinTm || 55),
        (this._modelPrimer.primerMaxTm = i.primerMaxTm || 70),
        (this._modelPrimer.anneal = new t.NASeq(n)),
        this._modelPrimer.anneal)
      )
        u && (this._modelPrimer.tm = u(n));
      else throw new Error("Unable to create seq");
      if (((this._modelPrimer.tail = new t.NASeq(a)), !this._modelPrimer.tail)) throw new Error("Unable to create seq");
      (Object.defineProperty(this, "seq", {
        get: function () {
          return this.getSeq();
        },
      }),
        Object.defineProperty(this, "anneal", {
          get: function () {
            return this._modelPrimer.anneal ? this._modelPrimer.anneal.wseq : null;
          },
          set: function (p) {
            if (((this._modelPrimer.anneal = new t.NASeq(p)), !this._modelPrimer.anneal)) throw new Error("Unable to create seq");
            ((this._modelPrimer.tm = null), (this._modelPrimer.ta = null));
          },
        }),
        Object.defineProperty(this, "tail", {
          get: function () {
            return this.getTail();
          },
          set: function (p) {
            this.setTail(p);
          },
        }),
        Object.defineProperty(this, "tm", {
          get: function () {
            return this._modelPrimer.tm;
          },
          set: function (p) {
            this._modelPrimer.tm = p;
          },
        }),
        Object.defineProperty(this, "ta", {
          get: function () {
            return this._modelPrimer.ta;
          },
          set: function (p) {
            this._modelPrimer.ta = p;
          },
        }),
        Object.defineProperty(this, "primerMonosalt", {
          get: function () {
            return this._modelPrimer.primerMonosalt;
          },
          set: function (p) {
            this._modelPrimer.primerMonosalt = p;
          },
        }),
        Object.defineProperty(this, "primerDisalt", {
          get: function () {
            return this._modelPrimer.primerDisalt;
          },
          set: function (p) {
            this._modelPrimer.primerDisalt = p;
          },
        }),
        Object.defineProperty(this, "primerCt", {
          get: function () {
            return this._modelPrimer.primerCt;
          },
          set: function (p) {
            this._modelPrimer.primerCt = p;
          },
        }),
        Object.defineProperty(this, "primerMethod", {
          get: function () {
            return this._modelPrimer.primerMethod;
          },
          set: function (p) {
            this._modelPrimer.primerMethod = p;
          },
        }),
        Object.defineProperty(this, "primerMinLen", {
          get: function () {
            return this._modelPrimer.primerMinLen;
          },
          set: function (p) {
            this._modelPrimer.primerMinLen = p;
          },
        }),
        Object.defineProperty(this, "primerMinTm", {
          get: function () {
            return this._modelPrimer.primerMinTm;
          },
          set: function (p) {
            this._modelPrimer.primerMinTm = p;
          },
        }),
        Object.defineProperty(this, "primerMaxTm", {
          get: function () {
            return this._modelPrimer.primerMaxTm;
          },
          set: function (p) {
            this._modelPrimer.primerMaxTm = p;
          },
        }));
    }),
    (e.Primer.prototype.getSeq = function () {
      return this._modelPrimer.tail.wseq + this._modelPrimer.anneal.wseq;
    }),
    (e.Primer.prototype.getTail = function () {
      return this._modelPrimer.tail.wseq;
    }),
    (e.Primer.prototype.setTail = function (n) {
      if (((this._modelPrimer.tail = new t.NASeq(n, { extended: !0 })), !this._modelPrimer.tail)) throw new Error("Unable to create seq");
    }),
    (e.Primer.prototype.setForward = function () {
      return ((this._modelPrimer.direction = "fwd"), this);
    }),
    (e.Primer.prototype.setReverse = function () {
      return ((this._modelPrimer.direction = "rev"), this);
    }),
    (e.Primer.prototype.direction = function () {
      return this._modelPrimer.direction;
    }),
    (e.Primer.prototype.toJson = function () {
      return this._modelPrimer;
    }),
    (e.Primer.prototype.deepCopy = function () {
      var n = this._modelPrimer,
        a = new e.BuilderPrimer(""),
        i = {};
      for (var u in n) {
        if (n[u] === null) {
          i[u] = null;
          continue;
        }
        if (n[u] === void 0) {
          i[u] = void 0;
          continue;
        }
        switch (u) {
          case "anneal":
          case "overlap":
          case "spacer":
          case "tail":
            i[u] = n[u].deepCopy();
            break;
          default:
            i[u] = n[u];
        }
      }
      return ((a._modelPrimer = i), a);
    }),
    (e.Primer.calculatePrimers = function (n, a, i) {
      var u, f, h, d, p;
      (a || (a = {}),
        a && a.primerMonosalt && a.primerCt && ((f = a.primerMonosalt), (h = a.primerCt)),
        a && a.primerMethod ? (u = a.primerMethod) : (u = 4),
        (f = f || 150),
        (h = h || 500),
        (i = i || e.createTmFunction(u, f, h)));
      var v = {},
        _ = n.length,
        b = a.primerMinLen || 20,
        w = a.primerMinLen || 20,
        y = b,
        A = w,
        q = t.revcomp(n);
      if (b > _) throw new Error("Frag (" + _ + " bp) is shorter  than min primer length (" + b + " bp)");
      if (b + w > _) throw new Error("Autogen primers will overlap.");
      ((d = a.primerMinTm || 55), (p = a.primerMaxTm || 70), (v.pfwd = n.slice(0, b)), (v.prev = q.slice(0, w)));
      var T = i(v.pfwd);
      for (v.tmfwd = T; T < d; ) {
        if (b >= _) throw new Error("cannot reach min Tm " + d + " for forward primer within " + _ + " bp");
        ((v.pfwd = n.slice(0, ++b)), (T = i(v.pfwd)), (v.tmfwd = T), (y = b));
      }
      for (; T < p && b < _ && v.pfwd.search("[GgCc]$") === -1; ) ((v.pfwd = n.slice(0, ++b)), (T = i(v.pfwd)), (v.tmfwd = T));
      v.pfwd.search("[GCgc]$") === -1 && ((v.pfwd = n.slice(0, y)), (T = i(v.pfwd)), (v.tmfwd = T), (b = y));
      var N = i(v.prev);
      for (v.tmrev = N; N < d; ) {
        if (w >= _) throw new Error("cannot reach min Tm " + d + " for reverse primer within " + _ + " bp");
        ((v.prev = q.slice(0, ++w)), (N = i(v.prev)), (v.tmrev = N), (A = w));
      }
      for (; N < p && w < _ && v.prev.search("[GgCc]$") === -1; ) ((v.prev = q.slice(0, ++w)), (N = i(v.prev)), (v.tmrev = N));
      if ((v.prev.search("[GgCc]$") === -1 && ((v.prev = q.slice(0, A)), (N = i(v.prev)), (v.tmrev = N), (w = A)), T > N)) {
        for (; T - N > 5 && !(w >= _); ) ((v.prev = q.slice(0, ++w)), (N = i(v.prev)), (v.tmrev = N));
        for (A = w; N < T && w < _ && v.prev.search("[GgCc]$") === -1; )
          if (((v.prev = q.slice(0, ++w)), (N = i(v.prev)), N < T)) A = w;
          else {
            ((v.prev = q.slice(0, A)), (N = i(v.prev)), (v.tmrev = N));
            break;
          }
      }
      if (N > T) {
        for (; N - T > 5 && !(b >= _); ) ((v.pfwd = n.slice(0, ++b)), (T = i(v.pfwd)), (v.tmfwd = T));
        for (y = b; T < N && b < _ && v.pfwd.search("[GgCc]$") === -1; )
          if (((v.pfwd = n.slice(0, ++b)), (T = i(v.pfwd)), (v.tmfwd = T), T < N)) y = b;
          else {
            ((v.pfwd = n.slice(0, y)), (T = i(v.pfwd)), (v.tmfwd = T));
            break;
          }
      }
      if (v.prev.length + v.pfwd.length > _) throw new Error("Autogen primers will overlap.");
      return v;
    }),
    (e.Primer.locate_primer = function (n, a, i, u, f) {
      ((i = i || this.primerMinLen), (u = u || this.primerMinTm), (f = f || 0), (n = n.toLowerCase()));
      var h = n.slice(-8),
        d = new RegExp(h, "gi"),
        p = [],
        v,
        _ = 0,
        b = n.length,
        w,
        y,
        A,
        q,
        T,
        N = "watson";
      for (y = b - i; (v = d.exec(a)) !== null; ) {
        for (_ = 0, T = v.index, w = b - h.length, q = w, A = T; _ <= f && A >= 0 && q >= 0; )
          if ((--A, --q, n[q] !== a[A] && (++_, _ > f))) {
            ((q += 1), (A += 1), (_ -= 1));
            break;
          }
        (q === -1 && (q = 0), A === -1 && (A = 0), q <= y && p.push({ ppos: q, tpos: A, mm: _, strand: N }), (d.lastIndex = v.index + 1));
      }
      for (N = "crick", a = t.revcomp(a), d.lastIndex = 0; (v = d.exec(a)) !== null; ) {
        for (_ = 0, T = v.index, w = b - h.length, q = w, A = T; _ <= f && A >= 0 && q >= 0; )
          if ((--A, --q, n[q] !== a[A] && (++_, _ > f))) {
            ((q += 1), (A += 1), (_ -= 1));
            break;
          }
        (q === -1 && (q = 0), A === -1 && (A = 0), q <= y && p.push({ ppos: q, tpos: A, mm: _, strand: N }), (d.lastIndex = v.index + 1));
      }
      return p;
    }),
    (e.BuilderPrimer = function (n, a, i, u) {
      if (
        ((a = a || ""),
        (i = i || ""),
        (u = u || {
          primerMethod: 4,
          bufferGroup: "Q5",
          primerMonosalt: 50,
          primerCt: 200,
          primerMinLen: 20,
          primerMinTm: 55,
          primerMaxTm: 70,
        }),
        e.Primer.call(this, n, i + a, u),
        (this._modelPrimer.spacer = new t.NASeq(a)),
        !this._modelPrimer.spacer)
      )
        throw new Error("Unable to create seq");
      if (((this._modelPrimer.overlap = new t.NASeq(i)), !this._modelPrimer.overlap)) throw new Error("Unable to create seq");
      (Object.defineProperty(this, "overlap", {
        get: function () {
          return this._modelPrimer.overlap.wseq || "";
        },
        set: function (f) {
          if (((this._modelPrimer.overlap = new t.NASeq(f)), !this._modelPrimer.overlap)) throw new Error("Unable to create seq");
        },
      }),
        Object.defineProperty(this, "spacer", {
          get: function () {
            return this._modelPrimer.spacer.wseq ? this._modelPrimer.spacer.wseq : "";
          },
          set: function (f) {
            if (((this._modelPrimer.spacer = new t.NASeq(f)), !this._modelPrimer.spacer)) throw new Error("Unable to create seq");
          },
        }));
    }),
    (e.BuilderPrimer.prototype = Object.create(e.Primer.prototype)),
    (e.BuilderPrimer.prototype.constructor = e.BuilderPrimer),
    (e.BuilderPrimer.prototype.getSeq = function () {
      return this.overlap + this.spacer + this.anneal;
    }),
    (e.BuilderPrimer.prototype.getTail = function () {
      return this.overlap + this.spacer;
    }),
    (e.BuilderPrimer.prototype.setTail = function (n) {
      if (((this._modelPrimer.overlap = new t.NASeq(n)), !this._modelPrimer.overlap)) throw new Error("Unable to create seq");
    }),
    (e.BuilderPrimer.prototype.updateTm = function (n) {
      (n && n.primerMethod && (this.primerMethod = n.primerMethod),
        n && n.primerCt && (this.primerCt = n.primerCt),
        n && n.primerMonosalt && (this.primerMonosalt = n.primerMonosalt));
      var a = e.createTmFunction(this.primerMethod, this.primerMonosalt, this.primerCt);
      this.tm = a(this.anneal);
    }),
    e
  );
})(uo);
const Wo = uo;
var fo = {};
(function (l) {
  var e = l,
    t = Te,
    s = Wo;
  return (
    (e.Fragment = function (n, a) {
      if (n == null) throw new Error("Missing argument");
      if (((this._modelFragment = {}), (this._modelFragment.srcseq = new t.NASeq(n)), !this._modelFragment.srcseq))
        throw new Error("Unable to create seq");
      if (
        ((this._modelFragment.subseq = [0, this._modelFragment.srcseq.wseq.length - 1]),
        (this._modelFragment.fwdPrimer = null),
        (this._modelFragment.revPrimer = null),
        (this._modelFragment.fwdTm = null),
        (this._modelFragment.revTm = null),
        (this._modelFragment.customFwdPrimer = !1),
        (this._modelFragment.customRevPrimer = !1),
        (this._modelFragment.leftRE = null),
        (this._modelFragment.rightRE = null),
        (this._modelFragment.leftREName = null),
        (this._modelFragment.rightREName = null),
        (this._modelFragment.leftREAdjustment = "none"),
        (this._modelFragment.rightREAdjustment = "none"),
        (this._modelFragment.upstreamJunction = null),
        (this._modelFragment.downstreamJunction = null),
        (this._modelFragment.upstreamJunctionID = null),
        (this._modelFragment.downstreamJunctionID = null),
        (this._modelFragment.leftInset = 0),
        (this._modelFragment.rightInset = 0),
        (this._modelFragment.id = null),
        (this._modelFragment.polID = "q5-0"),
        (this._modelFragment.polGroup = "Q5 High-Fidelity DNA Polymerase"),
        (this._modelFragment.primerMinLen = 15),
        (this._modelFragment.primerMethod = 4),
        (this._modelFragment.primerCt = 500),
        (this._modelFragment.primerMonosalt = 150),
        (this._modelFragment.primerMinTm = 55),
        (this._modelFragment.primerMaxTm = 70),
        (this._modelFragment.maxOverlapLength = 60),
        (this._modelFragment.digestIndex = 0),
        (this._modelFragment.isVectorBackbone = !1),
        (this._modelFragment.padseq = ""),
        a && a.srctopology)
      )
        switch (a.srctopology.toLowerCase()) {
          case "circular":
            this._modelFragment.srcseq.setSeqTopology("circular");
            break;
          case "linear":
            this._modelFragment.srcseq.setSeqTopology("linear");
            break;
          default:
            throw new Error("Invalid topology");
        }
      else this._modelFragment.srcseq.setSeqTopology("linear");
      if (a && a.srctype)
        switch (a.srctype.toLowerCase()) {
          case "synth":
          case "redig":
          case "pcr":
            this._modelFragment.srctype = a.srctype.toLowerCase();
            break;
          default:
            throw new Error("Invalid srctype");
        }
      else this._modelFragment.srctype = "synth";
      (a && a.srcname && (this._modelFragment.srcseq.sname = a.srcname),
        a && a.polID && (this._modelFragment.polID = a.polID),
        a && a.polGroup && (this._modelFragment.polGroup = a.polGroup),
        a && a.primerMethod && (this._modelFragment.primerMethod = a.primerMethod),
        a && a.primerCt && (this._modelFragment.primerCt = a.primerCt),
        a && a.primerMonosalt && (this._modelFragment.primerMonosalt = a.primerMonosalt),
        a && a.primerMinLen && (this._modelFragment.primerMinLen = a.primerMinLen),
        a && a.primerMinTm && (this._modelFragment.primerMinTm = a.primerMinTm),
        a && a.primerMaxTm && (this._modelFragment.primerMaxTm = a.primerMaxTm),
        a && a.isVectorBackbone && (this._modelFragment.isVectorBackbone = a.isVectorBackbone),
        Object.defineProperty(this, "id", {
          get: function () {
            return this._modelFragment.id;
          },
          set: function (i) {
            this._modelFragment.id = i;
          },
        }),
        Object.defineProperty(this, "isCircular", {
          get: function () {
            return this._modelFragment.srcseq.topology === "circular";
          },
          set: function (i) {
            i ? (this._modelFragment.srcseq.topology = "circular") : (this._modelFragment.srcseq.topology = "linear");
          },
        }),
        Object.defineProperty(this, "upstreamJunction", {
          get: function () {
            return this._modelFragment.upstreamJunction;
          },
          set: function (i) {
            this._modelFragment.upstreamJunction = i;
          },
        }),
        Object.defineProperty(this, "downstreamJunction", {
          get: function () {
            return this._modelFragment.downstreamJunction;
          },
          set: function (i) {
            this._modelFragment.downstreamJunction = i;
          },
        }),
        Object.defineProperty(this, "upstreamJunctionID", {
          get: function () {
            return this._modelFragment.upstreamJunctionID;
          },
          set: function (i) {
            this._modelFragment.upstreamJunctionID = i;
          },
        }),
        Object.defineProperty(this, "downstreamJunctionID", {
          get: function () {
            return this._modelFragment.downstreamJunctionID;
          },
          set: function (i) {
            this._modelFragment.downstreamJunctionID = i;
          },
        }),
        Object.defineProperty(this, "leftInset", {
          get: function () {
            return this._modelFragment.leftInset;
          },
          set: function (i) {
            this._modelFragment.leftInset = i;
          },
        }),
        Object.defineProperty(this, "rightInset", {
          get: function () {
            return this._modelFragment.rightInset;
          },
          set: function (i) {
            this._modelFragment.rightInset = i;
          },
        }),
        Object.defineProperty(this, "leftREAdjustment", {
          get: function () {
            return this._modelFragment.leftREAdjustment;
          },
          set: function (i) {
            this._modelFragment.leftREAdjustment = i;
          },
        }),
        Object.defineProperty(this, "rightREAdjustment", {
          get: function () {
            return this._modelFragment.rightREAdjustment;
          },
          set: function (i) {
            this._modelFragment.rightREAdjustment = i;
          },
        }),
        Object.defineProperty(this, "leftRE", {
          get: function () {
            return this._modelFragment.leftRE;
          },
          set: function (i) {
            this._modelFragment.leftRE = i;
          },
        }),
        Object.defineProperty(this, "rightRE", {
          get: function () {
            return this._modelFragment.rightRE;
          },
          set: function (i) {
            this._modelFragment.rightRE = i;
          },
        }),
        Object.defineProperty(this, "leftREName", {
          get: function () {
            return this._modelFragment.leftREName;
          },
          set: function (i) {
            this._modelFragment.leftREName = i;
          },
        }),
        Object.defineProperty(this, "rightREName", {
          get: function () {
            return this._modelFragment.rightREName;
          },
          set: function (i) {
            this._modelFragment.rightREName = i;
          },
        }),
        Object.defineProperty(this, "customFwdPrimer", {
          get: function () {
            return this._modelFragment.customFwdPrimer;
          },
          set: function (i) {
            this._modelFragment.customFwdPrimer = i;
          },
        }),
        Object.defineProperty(this, "customRevPrimer", {
          get: function () {
            return this._modelFragment.customRevPrimer;
          },
          set: function (i) {
            this._modelFragment.customRevPrimer = i;
          },
        }),
        Object.defineProperty(this, "fwdPrimer", {
          get: function () {
            return this._modelFragment.fwdPrimer;
          },
          set: function (i) {
            this._modelFragment.fwdPrimer = i;
          },
        }),
        Object.defineProperty(this, "revPrimer", {
          get: function () {
            return this._modelFragment.revPrimer;
          },
          set: function (i) {
            this._modelFragment.revPrimer = i;
          },
        }),
        Object.defineProperty(this, "fwdTm", {
          get: function () {
            return this._modelFragment.fwdTm;
          },
          set: function (i) {
            this._modelFragment.fwdTm = i;
          },
        }),
        Object.defineProperty(this, "revTm", {
          get: function () {
            return this._modelFragment.revTm;
          },
          set: function (i) {
            this._modelFragment.revTm = i;
          },
        }),
        Object.defineProperty(this, "fwdAnnealRange", {
          get: function () {
            return this._modelFragment.fwdAnnealRange;
          },
          set: function (i) {
            this._modelFragment.fwdAnnealRange = i;
          },
        }),
        Object.defineProperty(this, "revAnnealRange", {
          get: function () {
            return this._modelFragment.revAnnealRange;
          },
          set: function (i) {
            this._modelFragment.revAnnealRange = i;
          },
        }),
        Object.defineProperty(this, "srctype", {
          get: function () {
            return this._modelFragment.srctype;
          },
          set: function (i) {
            var u = this._modelFragment.srctype;
            ((this._modelFragment.srctype = i), u !== i && this.reset(this.primerSpecs));
          },
        }),
        Object.defineProperty(this, "src", {
          get: function () {
            return this._modelFragment.srcseq;
          },
          set: function (i) {
            if (((this._modelFragment.srcseq = new t.NASeq(i)), !this._modelFragment.srcseq)) throw new Error("Unable to create seq");
          },
        }),
        Object.defineProperty(this, "srcseq", {
          get: function () {
            return this._modelFragment.srcseq;
          },
          set: function (i) {
            if (((this._modelFragment.srcseq = new t.NASeq(i)), !this._modelFragment.srcseq)) throw new Error("Unable to create seq");
          },
        }),
        Object.defineProperty(this, "seq", {
          get: function () {
            return this.generateSeq(!1);
          },
        }),
        Object.defineProperty(this, "minseq", {
          get: function () {
            return this.generateSeq(!0);
          },
        }),
        Object.defineProperty(this, "subseq", {
          get: function () {
            return this._modelFragment.subseq;
          },
          set: function (i) {
            if (i.length !== 2) throw new Error("Invalid range input");
            this.setRange(i[0], i[1]);
          },
        }),
        Object.defineProperty(this, "template", {
          get: function () {
            var i = this.srcseq.subSeq(this.subseq[0], this.subseq[1] + 1);
            return ((i = this.padseq + i), (i = i.slice(this.leftInset, i.length - this.rightInset)), i);
          },
        }),
        Object.defineProperty(this, "polID", {
          get: function () {
            return this._modelFragment.polID;
          },
          set: function (i) {
            this._modelFragment.polID = i;
          },
        }),
        Object.defineProperty(this, "name", {
          get: function () {
            return this._modelFragment.srcseq.sname;
          },
          set: function (i) {
            this._modelFragment.srcseq.sname = i;
          },
        }),
        Object.defineProperty(this, "maxOverlapLength", {
          get: function () {
            return this._modelFragment.maxOverlapLength;
          },
          set: function (i) {
            this._modelFragment.maxOverlapLength = i;
          },
        }),
        Object.defineProperty(this, "primerMinLen", {
          get: function () {
            return this._modelFragment.primerMinLen;
          },
          set: function (i) {
            this._modelFragment.primerMinLen = i;
          },
        }),
        Object.defineProperty(this, "primerMinTm", {
          get: function () {
            return this._modelFragment.primerMinTm;
          },
          set: function (i) {
            this._modelFragment.primerMinTm = i;
          },
        }),
        Object.defineProperty(this, "primerMaxTm", {
          get: function () {
            return this._modelFragment.primerMaxTm;
          },
          set: function (i) {
            this._modelFragment.primerMaxTm = i;
          },
        }),
        Object.defineProperty(this, "primerMethod", {
          get: function () {
            return this._modelFragment.primerMethod;
          },
          set: function (i) {
            this._modelFragment.primerMethod = i;
          },
        }),
        Object.defineProperty(this, "primerDisalt", {
          get: function () {
            return this._modelFragment.primerDisalt;
          },
          set: function (i) {
            this._modelFragment.primerDisalt = i;
          },
        }),
        Object.defineProperty(this, "primerCt", {
          get: function () {
            return this._modelFragment.primerCt;
          },
          set: function (i) {
            this._modelFragment.primerCt = i;
          },
        }),
        Object.defineProperty(this, "primerMonosalt", {
          get: function () {
            return this._modelFragment.primerMonosalt;
          },
          set: function (i) {
            this._modelFragment.primerMonosalt = i;
          },
        }),
        Object.defineProperty(this, "digestIndex", {
          get: function () {
            return this._modelFragment.digestIndex;
          },
          set: function (i) {
            this._modelFragment.digestIndex = i;
          },
        }),
        Object.defineProperty(this, "isVectorBackbone", {
          get: function () {
            return this._modelFragment.isVectorBackbone;
          },
          set: function (i) {
            this._modelFragment.isVectorBackbone = !!i;
          },
        }),
        Object.defineProperty(this, "primerSpecs", {
          get: function () {
            return {
              polID: this._modelFragment.polID,
              primerMinLen: this._modelFragment.primerMinLen,
              polGroup: this._modelFragment.polGroup,
              primerMethod: this._modelFragment.primerMethod,
              primerCt: this._modelFragment.primerCt,
              primerMonosalt: this._modelFragment.primerMonosalt,
              primerMinTm: this._modelFragment.primerMinTm,
              primerMaxTm: this._modelFragment.primerMaxTm,
            };
          },
          set: function (i) {
            for (var u in i) i.hasOwnProperty(u) && (this._modelFragment[u] = i[u]);
          },
        }),
        Object.defineProperty(this, "padseq", {
          get: function () {
            return this._modelFragment.padseq;
          },
          set: function (i) {
            this._modelFragment.padseq = i;
          },
        }),
        this.reset(this.primerSpecs));
    }),
    (e.Fragment.prototype.deepCopy = function () {
      var n = this._modelFragment,
        a = new e.Fragment(""),
        i = {};
      for (var u in n) {
        if (n[u] === null) {
          i[u] = null;
          continue;
        }
        if (n[u] === void 0) {
          i[u] = void 0;
          continue;
        }
        switch (u) {
          case "srcseq":
            i.srcseq = n.srcseq.deepCopy();
            break;
          case "fwdPrimer":
            i.fwdPrimer = n.fwdPrimer.deepCopy();
            break;
          case "revPrimer":
            i.revPrimer = n.revPrimer.deepCopy();
            break;
          case "upstreamJunction":
            i.upstreamJunction = n.upstreamJunction.deepCopy();
            break;
          case "downstreamJunction":
            i.downstreamJunction = n.downstreamJunction.deepCopy();
            break;
          default:
            i[u] = n[u];
        }
      }
      return ((a._modelFragment = i), a);
    }),
    (e.Fragment.prototype.toJson = function () {
      return this._modelFragment;
    }),
    (e.Fragment.prototype.coord = function (n) {
      return this.srcseq.coord(n);
    }),
    (e.Fragment.prototype.generateSeq = function (n) {
      var a = n || !1,
        i = "",
        u = "",
        f,
        h;
      if (this.srctype === "synth") {
        if (this.subseq[0] < this.subseq[1])
          return this.srcseq.wseq.slice(this.subseq[0] + this.leftInset, this.subseq[1] + 1 - this.rightInset);
        if (this.srcseq.topology === "circular")
          return this.srcseq.wseq.slice(this.subseq[0] + this.leftInset) + this.srcseq.wseq.slice(0, this.subseq[1] + 1 - this.rightInset);
        throw new Error("subseq spans origin of linear src");
      } else if (this.srctype === "pcr") {
        if (this.fwdPrimer === null || this.revPrimer === null) {
          if (this.subseq[0] < this.subseq[1]) return this.srcseq.wseq.slice(this.subseq[0], this.subseq[1] + 1);
          if (this.srcseq.topology === "circular")
            return this.srcseq.wseq.slice(this.subseq[0]) + this.srcseq.wseq.slice(0, this.subseq[1] + 1);
          throw new Error("subseq spans origin of linear src");
        }
        if (((f = this.fwdPrimer), (h = this.revPrimer), this.fwdAnnealRange[1] < this.revAnnealRange[0]))
          u = this.srcseq.wseq.slice(this.fwdAnnealRange[1] + 1, this.revAnnealRange[0]);
        else if (this.srcseq.topology === "circular")
          u = this.srcseq.wseq.slice(this.fwdAnnealRange[1] + 1) + this.srcseq.wseq.slice(0, this.revAnnealRange[0]);
        else throw new Error("primers overlap - unable to generate ipr seq");
        return (
          a ? (i = f.anneal + u + t.revcomp(h.anneal)) : (i = f.seq + u + t.revcomp(h.seq)),
          (i = i.slice(this.leftInset, i.length - this.rightInset)),
          i
        );
      } else if (this.srctype === "redig")
        if (this.padseq === "") {
          if (this.subseq[0] < this.subseq[1])
            return this.srcseq.wseq.slice(this.subseq[0] + this.leftInset, this.subseq[1] + 1 - this.rightInset);
          if (this.srcseq.topology === "circular")
            return (
              this.srcseq.wseq.slice(this.subseq[0] + this.leftInset) + this.srcseq.wseq.slice(0, this.subseq[1] + 1 - this.rightInset)
            );
          throw new Error("subseq spans origin of linear src");
        } else {
          if (this.subseq[0] < this.subseq[1]) i = this.srcseq.wseq.slice(this.subseq[0], this.subseq[1] + 1);
          else if (this.srcseq.topology === "circular")
            i = this.srcseq.wseq.slice(this.subseq[0]) + this.srcseq.wseq.slice(0, this.subseq[1] + 1);
          else throw new Error("subseq spans origin of linear src");
          return ((i = this.padseq + i), i.slice(this.leftInset, i.length - this.rightInset));
        }
      return this.srcseq.wseq;
    }),
    (e.Fragment.prototype.setFwdPrimer = function (n, a, i, u) {
      if (((u = u || null), (i = !!i || !1), arguments.length < 2)) throw new Error("insufficient arguments");
      if (!(n instanceof s.BuilderPrimer)) throw new Error("invalid primer");
      if (Array.isArray(a)) {
        if (a.length !== 2) throw new Error("invalid range 2");
        if (a[0] < 0 || a[1] > this.srcseq.wseq.length) throw new Error("invalid range 3");
      } else throw new Error("invalid range 1");
      return ((this.customFwdPrimer = i), (this.fwdAnnealRange = a), (this.fwdPrimer = n), (this.fwdTa = u), (this.subseq[0] = a[0]), this);
    }),
    (e.Fragment.prototype.setRevPrimer = function (n, a, i, u) {
      if (((u = u || null), (i = !!i || !1), arguments.length < 2)) throw new Error("insufficient arguments");
      if (!(n instanceof s.BuilderPrimer)) throw new Error("invalid primer");
      if (Array.isArray(a)) {
        if (a.length !== 2) throw new Error("invalid range 2");
        if (a[0] < 0 || a[1] > this.srcseq.wseq.length) throw new Error("invalid range 3");
      } else throw new Error("invalid range 1");
      return ((this.customRevPrimer = i), (this.revAnnealRange = a), (this.revPrimer = n), (this.revTa = u), (this.subseq[1] = a[1]), this);
    }),
    (e.Fragment.prototype.getFwdPrimer = function () {
      return this.fwdPrimer;
    }),
    (e.Fragment.prototype.getRevPrimer = function () {
      return this.revPrimer;
    }),
    (e.Fragment.prototype.getFwdPrimerSeq = function () {
      return this.fwdPrimer.seq;
    }),
    (e.Fragment.prototype.getRevPrimerSeq = function () {
      return this.revPrimer.seq;
    }),
    (e.Fragment.prototype.setLeftRE = function (n, a, i, u) {
      var f = i || !1,
        h = u || !1,
        d = "";
      function p(T) {
        return T > 0 ? T - 1 : T;
      }
      var v, _, b;
      if (arguments.length < 2) throw new Error("insufficient arguments");
      if (a < 0 || a > this._modelFragment.srcseq.length) throw new Error("location out of range");
      n.c3 !== 0 ? ((_ = n.c3), (b = n.c4)) : ((_ = n.c1), (b = n.c2));
      var w = {};
      ((w.enzyme = n),
        (w.recognitionSite = a),
        (w.overhang3 = ""),
        (w.overhang5 = ""),
        (w.cutsite = a + p(_)),
        this.setStart(this.coord(w.cutsite + 1)),
        _ === b
          ? ((w.overhang3 = ""), (w.overhang5 = ""), (w.otype = "Blunt"))
          : _ > b
            ? ((v = a + p(b) + 1),
              (w.overhang3 = this.srcseq.subSeq(v, v + (_ - b))),
              (w.overhang5 = ""),
              (w.otype = "3-Prime"),
              (d = w.overhang3))
            : ((v = a + p(_) + 1), (w.overhang3 = ""), (w.overhang5 = this.srcseq.subSeq(v, v + (b - _))), (w.otype = "5-Prime")),
        (this._modelFragment.leftRE = w));
      var y = n.oseq,
        A = n.otype,
        A = n.otype,
        q = 0;
      return (
        y !== "" && A === "3-Prime" ? (q = -1 * y.length) : y !== "" && A === "5-Prime" && (q = y.length),
        f && this.adjustStart(q),
        h ? (this.padseq = d) : (this.padseq = ""),
        this
      );
    }),
    (e.Fragment.prototype.setRightRE = function (n, a) {
      function i(p) {
        return p > 0 ? p - 1 : p;
      }
      var u, f, h;
      if (arguments.length < 2) throw new Error("insufficient arguments");
      if (a < 0 || a > this._modelFragment.srcseq.length) throw new Error("location out of range");
      ((f = n.c1), (h = n.c2));
      var d = {};
      return (
        (d.enzyme = n),
        (d.recognitionSite = a),
        (d.overhang3 = ""),
        (d.overhang5 = ""),
        (d.cutsite = a + i(f)),
        this.setEnd(this.coord(d.cutsite)),
        f === h
          ? ((d.overhang3 = ""), (d.overhang5 = ""), (d.otype = "Blunt"))
          : f > h
            ? ((u = a + i(h) + 1), (d.overhang3 = this.srcseq.subSeq(u, u + (f - h))), (d.overhang5 = ""), (d.otype = "3-Prime"))
            : ((u = a + i(f) + 1), (d.overhang3 = ""), (d.overhang5 = this.srcseq.subSeq(u, u + (h - f))), (d.otype = "5-Prime")),
        (this._modelFragment.rightRE = d),
        this
      );
    }),
    (e.Fragment.prototype.rebuildRightRE = function (n, a, i) {
      ((n = n || "strict"), (a = a || ""));
      var u,
        f,
        h,
        d,
        p,
        v,
        _,
        b = this._modelFragment.rightRE,
        w = this._modelFragment.rightRE.enzyme.pat.toLowerCase(),
        y = b.recognitionSite,
        A = (y + w.length - 1) % this._modelFragment.srcseq.length,
        q;
      i ? (q = i.pat.length) : A > y ? (q = A - y) : (q = A - (y - this._modelFragment.srcseq.length));
      var T;
      i ? (T = [0, Number(i.c1), Number(i.c2), Number(i.c3), Number(i.c4)]) : (T = []);
      var N = this.subseq[1];
      if (n === "none") return "";
      if (!this.isCircular && N >= A) return "";
      if (this.isCircular && T[1] >= q - 1 && T[2] >= q - 1) return "";
      var M = this.srcseq.subSeq(N + 1, A + 1);
      if (n === "strict") return M;
      if (n === "min") {
        if (a.length < M.length) return M;
        for (h = w.length, d = M.length, u = 0, _ = 0; u < d; ) {
          for (f = 0, v = a[f], p = M[u + f]; Te.areEqualDNABases(v, p) && f < d - _; ) (++f, (v = a[f]), (p = M[u + f]));
          if (f === d - _) break;
          (++u, ++_);
        }
        return u === 0 ? "" : M.substring(0, u + 1);
      }
    }),
    (e.Fragment.prototype.rebuildLeftRE = function (n, a, i) {
      ((n = n || "strict"), (a = a || ""));
      var u,
        f,
        h,
        d,
        p,
        v,
        _,
        b,
        w = this._modelFragment.leftRE,
        y = w.recognitionSite,
        A = "",
        q;
      i ? (q = [0, Number(i.c1), Number(i.c2), Number(i.c3), Number(i.c4)]) : (q = []);
      var T = this._modelFragment.leftRE.enzyme.pat,
        N = (y + T.length - 1) % this._modelFragment.srcseq.length,
        M = this.subseq[0] - this.padseq.length;
      if (n === "none") return "";
      if (!this.isCircular && M < y) return "";
      if (this.isCircular && q[1] < 0 && q[2] < 0) return "";
      if (((A = this._modelFragment.srcseq.subSeq(y, M)), n === "strict")) return A;
      if (n === "min") {
        if (a.length < T.length) return A;
        for (h = T.length, d = A.length, u = d - 1, _ = 0, b = a.length - 1; u >= 0; ) {
          for (f = 0, v = a[b - f], p = A[u - f]; Te.areEqualDNABases(v, p) && f < d - _; ) (++f, (v = a[b - f]), (p = A[u - f]));
          if (f === d - _) break;
          (--u, ++_);
        }
        return u === d - 1 ? "" : A.slice(u + 1);
      }
    }),
    (e.Fragment.prototype.autogenPrimers = function (n) {
      if (
        (n || (n = {}),
        (n = Object.assign({}, n)),
        (n.polGroup = n.polGroup || "Q5"),
        (n.primerMethod = n.primerMethod || 4),
        (n.primerMonosalt = n.primerMonosalt || 150),
        (n.primerCt = n.primerCt || 500),
        (n.primerMinLen = n.primerMinLen || 15),
        (n.primerMinTm = n.primerMinTm || 55),
        (n.primerMaxTm = n.primerMaxTm || 70),
        (n.direction = n.direction || "both"),
        this.template.length < this.maxOverlapLength)
      )
        throw new Error("fragment too short for autogen primers");
      var a;
      try {
        a = s.Primer.calculatePrimers(this.template, n);
      } catch (b) {
        throw new Error("primer autogen failed: " + b.message);
      }
      var i = !1,
        u,
        f,
        h,
        d,
        p,
        v,
        _;
      ((_ = this.srcseq.wseq.length),
        (n.direction === "both" || n.direction === "fwd") &&
          ((u = a.pfwd),
          (f = (this.fwdPrimer && this.fwdPrimer.overlap) || ""),
          (h = (this.fwdPrimer && this.fwdPrimer.spacer) || ""),
          a.pfwd !== "" &&
            ((d = new s.BuilderPrimer(u, h, f, n)),
            (p = this._modelFragment.subseq[0]),
            (v = this._modelFragment.subseq[0] + a.pfwd.length - 1),
            v >= _ && (v -= _),
            this.setFwdPrimer(d, [p, v], i))),
        (n.direction === "both" || n.direction === "rev") &&
          ((u = a.prev),
          (f = (this.revPrimer && this.revPrimer.overlap) || ""),
          (h = (this.revPrimer && this.revPrimer.spacer) || ""),
          a.prev !== "" &&
            ((d = new s.BuilderPrimer(u, h, f, n)),
            (p = this._modelFragment.subseq[1] + 1 - a.prev.length),
            p < 0 && (p += _),
            (v = this._modelFragment.subseq[1]),
            this.setRevPrimer(new s.BuilderPrimer(u, h, f, n), [p, v], i))));
    }),
    (e.Fragment.prototype.autogenSDMPrimers = function (n) {
      if (
        (n || (n = {}),
        (n.polGroup = n.polGroup || "Q5"),
        (n.primerMethod = n.primerMethod || 4),
        (n.primerMonosalt = n.primerMonosalt || 150),
        (n.primerCt = n.primerCt || 500),
        (n.primerMinLen = n.primerMinLen || 15),
        (n.primerMinTm = n.primerMinTm || 55),
        (n.primerMaxTm = n.primerMaxTm || 70),
        (n.direction = n.direction || "both"),
        this.template.length < this.maxOverlapLength)
      )
        throw new Error("fragment too short for autogen primers");
      var a;
      try {
        a = s.Primer.calculatePrimers(this.template, n);
      } catch (b) {
        throw new Error("primer autogen failed: " + b.message);
      }
      var i = !1,
        u,
        f,
        h,
        d,
        p,
        v,
        _;
      ((_ = this.srcseq.wseq.length),
        (n.direction === "both" || n.direction === "fwd") &&
          ((u = a.pfwd),
          (f = (this.fwdPrimer && this.fwdPrimer.overlap) || ""),
          (h = (this.fwdPrimer && this.fwdPrimer.spacer) || ""),
          a.pfwd !== "" &&
            ((d = new s.BuilderPrimer(u, h, f, n)),
            (p = this._modelFragment.subseq[0]),
            (v = this._modelFragment.subseq[0] + a.pfwd.length - 1),
            v > _ && (v -= _),
            this.setFwdPrimer(d, [p, v], i))),
        (n.direction === "both" || n.direction === "rev") &&
          ((u = a.prev),
          (f = (this.revPrimer && this.revPrimer.overlap) || ""),
          (h = (this.revPrimer && this.revPrimer.spacer) || ""),
          a.prev !== "" &&
            ((d = new s.BuilderPrimer(u, h, f, n)),
            (p = this._modelFragment.subseq[1] + 1 - a.prev.length),
            p < 0 && (p += _),
            (v = this._modelFragment.subseq[1]),
            this.setRevPrimer(new s.BuilderPrimer(u, h, f, n), [p, v], i))));
    }),
    (e.Fragment.prototype.reset = function (n) {
      switch (this._modelFragment.srctype) {
        case "pcr":
          ((this._modelFragment.customFwdPrimer = !1), (this._modelFragment.customRevPrimer = !1), this.autogenPrimers(n));
          break;
        case "redig":
          ((this._modelFragment.leftRE = null),
            (this._modelFragment.rightRE = null),
            (this._modelFragment.leftREName = null),
            (this._modelFragment.rightREName = null));
          break;
      }
    }),
    (e.Fragment.prototype.setRange = function (n, a) {
      var i = [this._modelFragment.subseq[0], this._modelFragment.subseq[1]],
        u = this._modelFragment.srcseq.length,
        f = this._modelFragment.srcseq.topology;
      if (f === "linear" && (n > a || n < 0 || a < 0 || n > u || a > u))
        throw ((this._modelFragment.subseq = i), new Error("location out of range: negative or greater than length of linear fragment"));
      return (
        f == "circular" && ((n = n % u), (a = a % u), n < 0 && (n += u), a < 0 && (a += u)),
        (this._modelFragment.subseq = [n, a]),
        this
      );
    }),
    (e.Fragment.prototype.setStart = function (n) {
      var a = this._modelFragment.subseq[1];
      return (this.setRange(n, a), this);
    }),
    (e.Fragment.prototype.setEnd = function (n) {
      var a = this._modelFragment.subseq[0];
      return (this.setRange(a, n), this);
    }),
    (e.Fragment.prototype.adjustStart = function (n) {
      var a = this._modelFragment.subseq[0] + n,
        i = this._modelFragment.subseq[1];
      return (this.setRange(a, i), this);
    }),
    (e.Fragment.prototype.adjustEnd = function (n) {
      var a = this._modelFragment.subseq[0],
        i = this._modelFragment.subseq[1] + n;
      return (this.setRange(a, i), this);
    }),
    (e.Fragment.prototype.updateSrcseq = function (n) {
      this._modelFragment.srcseq.seq = n;
    }),
    (e.Fragment.prototype.clearPrimers = function (n) {
      switch (((n = n || "both"), n)) {
        case "fwd":
          this._modelFragment.fwdPrimer = null;
          break;
        case "rev":
          this._modelFragment.revPrimer = null;
          break;
        case "both":
          ((this._modelFragment.fwdPrimer = null), (this._modelFragment.revPrimer = null));
          break;
        default:
          ((this._modelFragment.fwdPrimer = null), (this._modelFragment.revPrimer = null));
      }
    }),
    (e.Fragment.prototype.updateTm = function (n) {
      if (this.srctype === "pcr") {
        n && (this.primerSpec = n);
        var a = { primerMethod: this.primerMethod, primerCt: this.primerCt, primerMonosalt: this.primerMonosalt };
        (this.fwdPrimer.updateTm(a), this.revPrimer.updateTm(a));
      }
    }),
    e
  );
})(fo);
const zo = fo;
var Ne = Ne || {};
Ne.trim = function (l) {
  return l.replace(/^\s+|\s+$/g, "");
};
Ne.revcomp = function (l) {
  var e = "ACGTUKMSWRYBDHVNacgtukmswrybdhvn -",
    t = "TGCAAMKSWYRVHDBNtgcaamkswyrvhdbn -",
    s = l.split("").reverse(),
    n,
    a;
  for (n = 0; n < s.length; ++n) {
    if (((a = e.indexOf(s[n])), a === -1)) throw { name: "InvalidCharacter", message: "A non IUPAC base was encountered: " + s[n] };
    s[n] = t.charAt(a);
  }
  return s.join("");
};
Ne.createTmCalc = function (l) {
  if (!l.seq) throw { name: "Missing seq", message: "A property named seq must be present in the object provided to createTmCalc" };
  var e = l.seq,
    t = l.ct || 0.25,
    s = l.salt || 50,
    n = l.dmso || 0,
    a = l.method || 1,
    i = l.compseq || null,
    u = Ne.trim(e.toLowerCase()).replace(/\s/g, "").replace(/u/g, "t"),
    f = i ? Ne.trim(i).toLowerCase().replace(/s/g, "") : Ne.revcomp(u),
    h = h || "",
    d,
    p,
    v,
    _ = 1.987,
    b = {
      aa: -24,
      tt: -24,
      at: -23.9,
      ta: -16.9,
      ca: -12.9,
      tg: -12.9,
      gt: -17.3,
      ac: -17.3,
      ct: -20.8,
      ag: -20.8,
      ga: -13.5,
      tc: -13.5,
      cg: -27.8,
      gc: -26.7,
      gg: -26.6,
      cc: -26.6,
    },
    w = {
      aa: -22.2,
      tt: -22.2,
      at: -20.4,
      ta: -21.3,
      ca: -22.7,
      tg: -22.7,
      gt: -22.4,
      ac: -22.4,
      ct: -21,
      ag: -21,
      ga: -22.2,
      tc: -22.2,
      cg: -27.2,
      gc: -24.4,
      gg: -19.9,
      cc: -19.9,
    },
    y = {
      aa: -9.1,
      tt: -9.1,
      at: -8.6,
      ta: -6,
      ca: -5.8,
      tg: -5.8,
      gt: -6.5,
      ac: -6.5,
      ct: -7.8,
      ag: -7.8,
      ga: -5.6,
      tc: -5.6,
      cg: -11.9,
      gc: -11.1,
      gg: -11,
      cc: -11,
    },
    A = {
      aa: -7.9,
      tt: -7.9,
      at: -7.2,
      ta: -7.2,
      ca: -8.5,
      tg: -8.5,
      gt: -8.4,
      ac: -8.4,
      ct: -7.8,
      ag: -7.8,
      ga: -8.2,
      tc: -8.2,
      cg: -10.6,
      gc: -9.8,
      gg: -8,
      cc: -8,
    },
    q = {
      "-a/at": { dh: -0.7, ds: -0.8 },
      "-a/ct": { dh: 4.4, ds: 14.9 },
      "-a/gt": { dh: -1.6, ds: -3.6 },
      "-a/tt": { dh: 2.9, ds: 10.4 },
      "-c/ag": { dh: -2.1, ds: -3.9 },
      "-c/cg": { dh: -0.2, ds: -0.1 },
      "-c/gg": { dh: -3.9, ds: -11.2 },
      "-c/tg": { dh: -4.4, ds: -13.1 },
      "-g/ac": { dh: -5.9, ds: -16.5 },
      "-g/cc": { dh: -2.6, ds: -7.4 },
      "-g/gc": { dh: -3.2, ds: -10.4 },
      "-g/tc": { dh: -5.2, ds: -15 },
      "-t/aa": { dh: -0.5, ds: -1.1 },
      "-t/ca": { dh: 4.7, ds: 7.7 },
      "-t/ga": { dh: -4.1, ds: -13.1 },
      "-t/ta": { dh: -3.8, ds: -12.6 },
      "a-/ta": { dh: -2.9, ds: -7.6 },
      "a-/tc": { dh: -4.1, ds: -13 },
      "a-/tg": { dh: -4.2, ds: -15 },
      "a-/tt": { dh: -0.2, ds: -0.5 },
      "aa/-t": { dh: 0.2, ds: 2.3 },
      "aa/at": { dh: 4.7, ds: 12.9 },
      "aa/ct": { dh: 7.6, ds: 20.2 },
      "aa/gt": { dh: 3, ds: 7.4 },
      "aa/t-": { dh: -0.5, ds: -1.1 },
      "aa/ta": { dh: 1.2, ds: 1.7 },
      "aa/tc": { dh: 2.3, ds: 4.6 },
      "aa/tg": { dh: -0.6, ds: -2.3 },
      "aa/tt": { dh: -7.6, ds: -21.3 },
      "ac/-g": { dh: -6.3, ds: -17.1 },
      "ac/ag": { dh: -2.9, ds: -9.8 },
      "ac/cg": { dh: -0.7, ds: -3.8 },
      "ac/gg": { dh: 0.5, ds: 3.2 },
      "ac/t-": { dh: 4.7, ds: 7.7 },
      "ac/ta": { dh: 5.3, ds: 14.6 },
      "ac/tc": { dh: 0, ds: -4.4 },
      "ac/tg": { dh: -8.4, ds: -22.4 },
      "ac/tt": { dh: 0.7, ds: 0.2 },
      "ag/-c": { dh: -3.7, ds: -10 },
      "ag/ac": { dh: -0.9, ds: -4.2 },
      "ag/cc": { dh: 0.6, ds: -0.6 },
      "ag/gc": { dh: -4, ds: -13.2 },
      "ag/t-": { dh: -4.1, ds: -13.1 },
      "ag/ta": { dh: -0.7, ds: -2.3 },
      "ag/tc": { dh: -7.8, ds: -21 },
      "ag/tg": { dh: -3.1, ds: -9.5 },
      "ag/tt": { dh: 1, ds: 0.9 },
      "at/-a": { dh: -2.9, ds: -7.6 },
      "at/aa": { dh: 1.2, ds: 1.7 },
      "at/ca": { dh: 5.3, ds: 14.6 },
      "at/ga": { dh: -0.7, ds: -2.3 },
      "at/t-": { dh: -3.8, ds: -12.6 },
      "at/ta": { dh: -7.2, ds: -20.4 },
      "at/tc": { dh: -1.2, ds: -6.2 },
      "at/tg": { dh: -2.5, ds: -8.3 },
      "at/tt": { dh: -2.7, ds: -10.8 },
      "c-/ga": { dh: -3.7, ds: -10 },
      "c-/gc": { dh: -4, ds: -11.9 },
      "c-/gg": { dh: -3.9, ds: -10.9 },
      "c-/gt": { dh: -4.9, ds: -13.8 },
      "ca/-t": { dh: 0.6, ds: 3.3 },
      "ca/at": { dh: 3.4, ds: 8 },
      "ca/ct": { dh: 6.1, ds: 16.4 },
      "ca/g-": { dh: -5.9, ds: -16.5 },
      "ca/ga": { dh: -0.9, ds: -4.2 },
      "ca/gc": { dh: 1.9, ds: 3.7 },
      "ca/gg": { dh: -0.7, ds: -2.3 },
      "ca/gt": { dh: -8.5, ds: -22.7 },
      "ca/tt": { dh: 1, ds: 0.7 },
      "cc/-g": { dh: -4.4, ds: -12.6 },
      "cc/ag": { dh: 5.2, ds: 14.2 },
      "cc/cg": { dh: 3.6, ds: 8.9 },
      "cc/g-": { dh: -2.6, ds: -7.4 },
      "cc/ga": { dh: 0.6, ds: -0.6 },
      "cc/gc": { dh: -1.5, ds: -7.2 },
      "cc/gg": { dh: -8, ds: -19.9 },
      "cc/gt": { dh: -0.8, ds: -4.5 },
      "cc/tg": { dh: 5.2, ds: 13.5 },
      "cg/-c": { dh: -4, ds: -11.9 },
      "cg/ac": { dh: 1.9, ds: 3.7 },
      "cg/cc": { dh: -1.5, ds: -7.2 },
      "cg/g-": { dh: -3.2, ds: -10.4 },
      "cg/ga": { dh: -4, ds: -13.2 },
      "cg/gc": { dh: -10.6, ds: -27.2 },
      "cg/gg": { dh: -4.9, ds: -15.3 },
      "cg/gt": { dh: -4.1, ds: -11.7 },
      "cg/tc": { dh: -1.5, ds: -6.1 },
      "ct/-a": { dh: -4.1, ds: -13 },
      "ct/aa": { dh: 2.3, ds: 4.6 },
      "ct/ca": { dh: 0, ds: -4.4 },
      "ct/g-": { dh: -5.2, ds: -15 },
      "ct/ga": { dh: -7.8, ds: -21 },
      "ct/gc": { dh: -1.5, ds: -6.1 },
      "ct/gg": { dh: -2.8, ds: -8 },
      "ct/gt": { dh: -5, ds: -15.8 },
      "ct/ta": { dh: -1.2, ds: -6.2 },
      "g-/ca": { dh: -6.3, ds: -17.1 },
      "g-/cc": { dh: -4.4, ds: -12.6 },
      "g-/cg": { dh: -5.1, ds: -14 },
      "g-/ct": { dh: -4, ds: -10.9 },
      "ga/-t": { dh: -1.1, ds: -1.6 },
      "ga/at": { dh: 0.7, ds: 0.7 },
      "ga/c-": { dh: -2.1, ds: -3.9 },
      "ga/ca": { dh: -2.9, ds: -9.8 },
      "ga/cc": { dh: 5.2, ds: 14.2 },
      "ga/cg": { dh: -0.6, ds: -1 },
      "ga/ct": { dh: -8.2, ds: -22.2 },
      "ga/gt": { dh: 1.6, ds: 3.6 },
      "ga/tt": { dh: -1.3, ds: -5.3 },
      "gc/-g": { dh: -5.1, ds: -14 },
      "gc/ag": { dh: -0.6, ds: -1 },
      "gc/c-": { dh: -0.2, ds: -0.1 },
      "gc/ca": { dh: -0.7, ds: -3.8 },
      "gc/cc": { dh: 3.6, ds: 8.9 },
      "gc/cg": { dh: -9.8, ds: -24.4 },
      "gc/ct": { dh: 2.3, ds: 5.4 },
      "gc/gg": { dh: -6, ds: -15.8 },
      "gc/tg": { dh: -4.4, ds: -12.3 },
      "gg/-c": { dh: -3.9, ds: -10.9 },
      "gg/ac": { dh: -0.7, ds: -2.3 },
      "gg/c-": { dh: -3.9, ds: -11.2 },
      "gg/ca": { dh: 0.5, ds: 3.2 },
      "gg/cc": { dh: -8, ds: -19.9 },
      "gg/cg": { dh: -6, ds: -15.8 },
      "gg/ct": { dh: 3.3, ds: 10.4 },
      "gg/gc": { dh: -4.9, ds: -15.3 },
      "gg/tc": { dh: -2.8, ds: -8 },
      "gt/-a": { dh: -4.2, ds: -15 },
      "gt/aa": { dh: -0.6, ds: -2.3 },
      "gt/c-": { dh: -4.4, ds: -13.1 },
      "gt/ca": { dh: -8.4, ds: -22.4 },
      "gt/cc": { dh: 5.2, ds: 13.5 },
      "gt/cg": { dh: -4.4, ds: -12.3 },
      "gt/ct": { dh: -2.2, ds: -8.4 },
      "gt/ga": { dh: -3.1, ds: -9.5 },
      "gt/ta": { dh: -2.5, ds: -8.3 },
      "t-/aa": { dh: 0.2, ds: 2.3 },
      "t-/ac": { dh: 0.6, ds: 3.3 },
      "t-/ag": { dh: -1.1, ds: -1.6 },
      "t-/at": { dh: -6.9, ds: -20 },
      "ta/-t": { dh: -6.9, ds: -20 },
      "ta/a-": { dh: -0.7, ds: -0.8 },
      "ta/aa": { dh: 4.7, ds: 12.9 },
      "ta/ac": { dh: 3.4, ds: 8 },
      "ta/ag": { dh: 0.7, ds: 0.7 },
      "ta/at": { dh: -7.2, ds: -21.3 },
      "ta/ct": { dh: 1.2, ds: 0.7 },
      "ta/gt": { dh: -0.1, ds: -1.7 },
      "ta/tt": { dh: 0.2, ds: -1.5 },
      "tc/-g": { dh: -4, ds: -10.9 },
      "tc/a-": { dh: 4.4, ds: 14.9 },
      "tc/aa": { dh: 7.6, ds: 20.2 },
      "tc/ac": { dh: 6.1, ds: 16.4 },
      "tc/ag": { dh: -8.2, ds: -22.2 },
      "tc/at": { dh: 1.2, ds: 0.7 },
      "tc/cg": { dh: 2.3, ds: 5.4 },
      "tc/gg": { dh: 3.3, ds: 10.4 },
      "tc/tg": { dh: -2.2, ds: -8.4 },
      "tg/-c": { dh: -4.9, ds: -13.8 },
      "tg/a-": { dh: -1.6, ds: -3.6 },
      "tg/aa": { dh: 3, ds: 7.4 },
      "tg/ac": { dh: -8.5, ds: -22.7 },
      "tg/ag": { dh: 1.6, ds: 3.6 },
      "tg/at": { dh: -0.1, ds: -1.7 },
      "tg/cc": { dh: -0.8, ds: -4.5 },
      "tg/gc": { dh: -4.1, ds: -11.7 },
      "tg/tc": { dh: -5, ds: -15.8 },
      "tt/-a": { dh: -0.2, ds: -0.5 },
      "tt/a-": { dh: 2.9, ds: 10.4 },
      "tt/aa": { dh: -7.6, ds: -21.3 },
      "tt/ac": { dh: 1, ds: 0.7 },
      "tt/ag": { dh: -1.3, ds: -5.3 },
      "tt/at": { dh: 0.2, ds: -1.5 },
      "tt/ca": { dh: 0.7, ds: 0.2 },
      "tt/ga": { dh: 1, ds: 0.9 },
      "tt/ta": { dh: -2.7, ds: -10.8 },
    },
    T = {
      3: { dh: 0, ds: -10.3 },
      4: { dh: 0, ds: -11.6 },
      5: { dh: 0, ds: -12.9 },
      6: { dh: 0, ds: -14.2 },
      7: { dh: 0, ds: -14.8 },
      8: { dh: 0, ds: -15.5 },
      9: { dh: 0, ds: -15.8 },
      10: { dh: 0, ds: -15.8 },
      12: { dh: 0, ds: -16.8 },
      14: { dh: 0, ds: -17.4 },
      16: { dh: 0, ds: -18.1 },
      18: { dh: 0, ds: -18.7 },
      20: { dh: 0, ds: -19 },
      25: { dh: 0, ds: -20.3 },
      30: { dh: 0, ds: -21.3 },
    },
    N = {
      1: { dh: 0, ds: -12.9 },
      2: { dh: 0, ds: -9.4 },
      3: { dh: 0, ds: -10 },
      4: { dh: 0, ds: -10.3 },
      5: { dh: 0, ds: -10.6 },
      6: { dh: 0, ds: -11.3 },
      7: { dh: 0, ds: -11.9 },
      8: { dh: 0, ds: -12.6 },
      9: { dh: 0, ds: -13.2 },
      10: { dh: 0, ds: -13.9 },
      12: { dh: 0, ds: -14.5 },
      14: { dh: 0, ds: -15.5 },
      16: { dh: 0, ds: -16.1 },
      18: { dh: 0, ds: -16.8 },
      20: { dh: 0, ds: -17.1 },
      25: { dh: 0, ds: -18.1 },
      30: { dh: 0, ds: -19 },
    },
    M = 0,
    k,
    E,
    P,
    G,
    O,
    K = function () {
      var U,
        le = this.wseq.split(""),
        J = this.cseq.split("").reverse(),
        ie;
      for (U = 0; U < le.length; ++U)
        le[U] === Ne.comp(J[U]) ? (this.pairmap += "|") : le[U] === "-" || J[U] === "-" ? (this.pairmap += "^") : (this.pairmap += "x");
      for (ie = this.pairmap.split(""), U = 1; U < ie.length; ++U)
        ie[U] === "x" && (ie[U - 1] === "x" || ie[U - 1] === "_") && ((ie[U] = "_"), (ie[U - 1] = "_"));
      for (this.pairmap = ie.join(""); this.pairmap.search(/\^+_+|_+\^+/) !== -1; )
        this.pairmap = this.pairmap.replace(/\^+_+|_+\^+/, function (z, te, pe) {
          pe.slice(0, te);
          var Y = new Array(z.length + 1).join("_");
          return (pe.slice(te + z.length), Y);
        });
      for (; this.pairmap.search(/\^+x|x\^+/) !== -1; )
        this.pairmap = this.pairmap.replace(/\^+x|x\^+/, function (z, te, pe) {
          pe.slice(0, te);
          var Y = new Array(z.length + 1).join("_");
          return (pe.slice(te + z.length), Y);
        });
      for (; this.pairmap.search(/_+[x^]|[x^]_+/) !== -1; )
        this.pairmap = this.pairmap.replace(/_+[x^]|[x^]_+/, function (z, te, pe) {
          pe.slice(0, te);
          var Y = new Array(z.length + 1).join("_");
          return (pe.slice(te + z.length), Y);
        });
    },
    D = function () {
      ((this.sc_sch = (16.6 * Math.log(this.saltc)) / Math.LN10),
        (this.sc_sl = 0.368 * this.wseq.length * Math.log(this.saltc)),
        (this.sc_ow = (4.29 * this.fgc - 3.95) * 1e-5 * Math.log(this.saltc) + 94e-7 * Math.log(this.saltc) * Math.log(this.saltc)));
    },
    L = function (U) {
      return ((this.method = U), this);
    },
    R = function (U, le) {
      var J;
      for (
        this.seq = U,
          this.wseq = Ne.trim(this.seq.toLowerCase().replace(/\s/g, "")),
          le ? (this.cseq = Ne.trim(le.toLowerCase().replace(/\s/g, ""))) : (this.cseq = Ne.revcomp(this.wseq)),
          this.sym = this.wseq === this.cseq,
          this.gc_cnt = 0,
          J = 0;
        J < this.wseq.length;
        ++J
      )
        (this.wseq.charAt(J) === "g" || this.wseq.charAt(J) === "c") && (this.gc_cnt += 1);
      return ((this.fgc = this.gc_cnt / this.wseq.length), this.saltCorrect(), this);
    },
    Q = function (U) {
      return ((this.ct = U), (this.primerc = t * 1e-6), this);
    },
    ae = function (U) {
      return ((this.salt = U), (this.saltc = this.salt * 0.001), this.saltCorrect(), this);
    },
    W = function (U) {
      return ((this.dmso = U), this);
    },
    x = function () {
      var U = 0,
        le = 0,
        J = 0,
        ie = 0,
        z = 0,
        te = 0,
        pe = this.primerc,
        Y,
        ue,
        he = this.wseq.split(""),
        de = this.cseq.split("").reverse(),
        _e;
      switch (
        (this.pairmap === "" &&
          (this.buildPairMap(),
          (he = this.wseq.split("")),
          (de = this.cseq.split("").reverse()),
          Ne.dmsg(
            this.wseq +
              `
` +
              this.pairmap +
              `
` +
              Ne.rev(this.cseq),
          )),
        (_e = this.pairmap.split("")),
        this.method)
      ) {
        case 1:
          for (ie += this.sym ? -12.4 : -10.8, Y = 0; Y < this.wseq.length - 1; ++Y)
            ((ue = this.wseq.slice(Y, Y + 2)), (le += this.dSBr[ue]), (J += this.dHBr[ue]));
          ((J *= 1e3), (U = J / (ie + le + this.R * Math.log(pe)) - 273.15 + this.sc_sch));
          break;
        case 3:
        case 4:
          for (
            te = this.sym ? -1.4 : 0,
              ie = -5.7,
              z = 200,
              (he[0] === "a" || he[0] === "t") && ((ie += 6.9), (z += 2200)),
              (he[u.length - 1] === "a" || he[this.wseq.length - 1] === "t") && ((ie += 6.9), (z += 2200)),
              Y = 0;
            Y < this.wseq.length - 1;
            Y++
          )
            (_e[Y] === "|" || _e[Y] === "x") &&
              (_e[Y + 1] === "|" || _e[Y + 1] === "x") &&
              ((ue = he[Y] + he[Y + 1] + "/" + de[Y] + de[Y + 1]),
              q.hasOwnProperty(ue) && ((le += this.nndhds[ue].ds), (J += this.nndhds[ue].dh)));
          ((U = (J * 1e3 + z) / (ie + te + le + this.R * Math.log(pe))), Ne.dmsg("tm of non-looped portion: " + U));
          var ne,
            $,
            fe = !1;
          ((fe = !1), (ne = 0), ($ = 0));
          var we, j;
          for (Y = 0; Y < _e.length; ++Y)
            if (_e[Y] === "_") {
              if ((he[Y] !== "-" && (ne += 1), de[Y] !== "-" && ($ += 1), !fe)) {
                for (fe = !0, we = Y; he[we] === "-"; ) ++we;
                for (j = Y; de[j] === "-"; ) ++j;
                ((le += this.nndhds[he[Y - 1] + "-/" + de[Y - 1] + de[j]].ds),
                  (J += this.nndhds[he[Y - 1] + "-/" + de[Y - 1] + de[j]].dh),
                  (le += this.nndhds[he[Y - 1] + he[we] + "/" + de[Y - 1] + "-"].ds),
                  (J += this.nndhds[he[Y - 1] + he[we] + "/" + de[Y - 1] + "-"].dh));
              }
            } else {
              if (ne + $ > 0) {
                for (
                  Ne.dmsg("loop: " + (ne + $)), le += this.loop["" + (ne + $)].ds, le -= Math.abs(ne - $) * 0.97, we = Y - 1;
                  he[we] === "-";
                )
                  --we;
                for (j = Y - 1; de[j] === "-"; ) --j;
                ((le += this.nndhds["-" + he[Y] + "/" + de[j] + de[Y]].ds),
                  (J += this.nndhds["-" + he[Y] + "/" + de[j] + de[Y]].dh),
                  (le += this.nndhds[he[we] + he[Y] + "/-" + de[Y]].ds),
                  (J += this.nndhds[he[we] + he[Y] + "/-" + de[Y]].dh));
              }
              ((ne = 0), ($ = 0), (fe = !1));
            }
          var F = 0;
          for (Y = 0; Y < _e.length; ++Y)
            _e[Y] === "^"
              ? (F += 1)
              : (F === 1 && Y >= 2
                  ? ((le += this.bulge["" + F].ds),
                    (le += this.nndhds[he[Y - 2] + he[Y] + "/" + de[Y - 2] + de[Y]].ds),
                    (J += this.nndhds[he[Y - 2] + he[Y] + "/" + de[Y - 2] + de[Y]].dh))
                  : F > 1 && (le += this.bulge["" + F].ds),
                (F = 0));
          ((J *= 1e3),
            (U = (J + z) / (ie + te + le + this.R * Math.log(pe))),
            this.method === 3 ? (U = 1 / (1 / U + this.sc_sl / (J + z))) : this.method === 4 && (U = 1 / (1 / U + this.sc_ow)),
            (U -= 273.15));
          break;
        case 7:
          U = 81.5 + (16.6 * Math.log(this.saltc)) / Math.LN10 + 0.41 * this.fgc - 675 / this.wseq.length;
          break;
      }
      return (
        (U = U - this.dmso * 0.6),
        Ne.dmsg("Tm is " + U),
        { method: this.method, wseq: this.wseq, cseq: this.cseq, tm: U, dh: J, ds: le, salt: this.saltc, ct: this.primerc, dmso: this.dmso }
      );
    };
  ((d = u === f), (p = t * 1e-6), (v = s * 0.001));
  var H = u.replace(/-/g, "");
  for (O = 0; O < H.length; ++O) (H.charAt(O) === "g" || H.charAt(O) === "c") && (M += 1);
  return (
    (k = M / H.length),
    (E = (16.6 * Math.log(v)) / Math.LN10),
    (P = 0.368 * u.length * Math.log(v)),
    (G = (4.29 * k - 3.95) * 1e-5 * Math.log(v) + 94e-7 * Math.log(v) * Math.log(v)),
    {
      setSeq: R,
      setSalt: ae,
      setCt: Q,
      setDMSO: W,
      setMethod: L,
      saltCorrect: D,
      Tm: x,
      gc_cnt: M,
      fgc: k,
      sc_sch: E,
      sc_sl: P,
      sc_ow: G,
      seq: e,
      compseq: i,
      ct: t,
      salt: s,
      dmso: n,
      method: a,
      wseq: u,
      cseq: f,
      sym: d,
      pairmap: h,
      buildPairMap: K,
      primerc: p,
      saltc: v,
      R: _,
      dSSa: w,
      dHSa: A,
      dSBr: b,
      dHBr: y,
      nndhds: q,
      loop: T,
      bulge: N,
    }
  );
};
Ne.isValidSeq = function (l, e, t) {
  var s = " acgt";
  s += s.toUpperCase();
  var n = " acgtwsrymkbdhvn";
  n += n.toUpperCase();
  var a, i;
  for (e === !1 ? (a = s) : (a = n), t === !0 && (a += "-"), i = 0; i < l.length; ++i) if (a.indexOf(l.charAt(i)) === -1) return !1;
  return !0;
};
Ne.dmsg = function (l) {};
var po = {};
(function (l) {
  var e = l;
  return (
    (e.alignDNAglobal = function (t) {
      var s = t.param || {},
        n = t.fullSeq1 || "",
        a = t.fullSeq2 || "",
        i,
        u,
        f,
        h,
        d = [],
        p = [],
        v,
        _,
        b,
        w,
        y = s.gap || -5,
        A = s.term || -1,
        q = s.extend || -1,
        T = [],
        N,
        M = [],
        k = s.mat || {
          a: { a: 3, c: -3, g: -3, t: -3 },
          c: { a: -3, c: 3, g: -3, t: -3 },
          g: { a: -3, c: -3, g: 3, t: -3 },
          t: { a: -3, c: -3, g: -3, t: 3 },
        };
      if (n == "" || a == "") return t;
      for (i = ("-" + n.toLowerCase()).split(""), u = ("-" + a.toLowerCase()).split(""), b = i.length, w = u.length, v = 0; v < b; ++v)
        for (d[v] = [], p[v] = [], _ = 0; _ < w; ++_) ((d[v][_] = 0), (p[v][_] = []));
      for (d[1][0] = y, p[1][0].push(1), v = 2; v < b; ++v) ((d[v][0] = v * A), p[v][0].push(1));
      for (d[0][1] = y, p[0][1].push(2), _ = 2; _ < w; ++_) ((d[0][_] = _ * A), p[0][_].push(2));
      for (v = 1; v < b; ++v)
        for (_ = 1; _ < w; ++_)
          ((T[0] = d[v - 1][_ - 1] + k[i[v]][u[_]]),
            p[v - 1][_].indexOf(1) != -1 ? (T[1] = d[v - 1][_] + q) : (T[1] = d[v - 1][_] + y),
            p[v][_ - 1].indexOf(2) != -1 ? (T[2] = d[v][_ - 1] + q) : (T[2] = d[v][_ - 1] + y),
            (N = Math.max(T[0], T[1], T[2])),
            (d[v][_] = N),
            N === T[0] && p[v][_].push(0),
            N === T[1] && p[v][_].push(1),
            N === T[2] && p[v][_].push(2));
      for (v = b - 1, _ = w - 1, f = "", h = ""; p[v][_].length != 0; )
        switch (p[v][_][p[v][_].length - 1]) {
          case 0:
            ((f = i[v--] + f), (h = u[_--] + h));
            break;
          case 1:
            ((f = i[v--] + f), (h = "-" + h));
            break;
          case 2:
            ((h = u[_--] + h), (f = "-" + f));
            break;
        }
      var E, P, G, O;
      return ((E = v), (G = _), (P = M[0] - 1), (O = M[1] - 1), t.setAlignedSeqs(f, E, P, h, G, O, e.buildAlignMap(f, h)), t);
    }),
    (e.alignDNAlocal = function (t) {
      t.param || (t.param = {});
      var s,
        n,
        a,
        i,
        u = [],
        f = [],
        h,
        d,
        p,
        v,
        _ = t.param.gap || -12,
        b = t.param.extend || -1,
        w = [],
        y,
        A = 0,
        q = [0, 0],
        T = 2,
        N = -4,
        M = t.param.mat || {
          a: { a: T, c: N, g: N, t: N },
          c: { a: N, c: T, g: N, t: N },
          g: { a: N, c: N, g: T, t: N },
          t: { a: N, c: N, g: N, t: T },
        };
      for (
        s = ("-" + t.fullSeq1.toLowerCase()).split(""), n = ("-" + t.fullSeq2.toLowerCase()).split(""), p = s.length, v = n.length, h = 0;
        h < p;
        ++h
      )
        for (u[h] = [], f[h] = [], d = 0; d < v; ++d) ((u[h][d] = 0), (f[h][d] = []));
      for (u[1][0] = 0, f[1][0].push(1), h = 2; h < p; ++h) ((u[h][0] = 0), f[h][0].push(-1));
      for (u[0][1] = 0, f[0][1].push(2), d = 2; d < v; ++d) ((u[0][d] = 0), f[0][d].push(-1));
      for (h = 1; h < p; ++h)
        for (d = 1; d < v; ++d)
          ((w[0] = u[h - 1][d - 1] + M[s[h]][n[d]]),
            f[h - 1][d].indexOf(1) != -1 ? (w[1] = u[h - 1][d] + b) : (w[1] = u[h - 1][d] + _),
            f[h][d - 1].indexOf(2) != -1 ? (w[2] = u[h][d - 1] + b) : (w[2] = u[h][d - 1] + _),
            (y = Math.max(w[0], w[1], w[2], 0)),
            (u[h][d] = y),
            u[h][d] > A && ((A = u[h][d]), (q = [h, d])),
            y === 0 && f[h][d].push(-1),
            y === w[0] && f[h][d].push(0),
            y === w[1] && f[h][d].push(1),
            y === w[2] && f[h][d].push(2),
            y === 0 && f[h][d].push(-1));
      ((h = q[0]), (d = q[1]), (a = ""), (i = ""));
      e: for (; f[h][d].length != 0; )
        switch (f[h][d][f[h][d].length - 1]) {
          case 0:
            ((a = s[h--] + a), (i = n[d--] + i));
            break;
          case 1:
            ((a = s[h--] + a), (i = "-" + i));
            break;
          case 2:
            ((i = n[d--] + i), (a = "-" + a));
            break;
          case -1:
            break e;
        }
      var k, E, P, G;
      return ((k = h), (P = d), (E = q[0] - 1), (G = q[1] - 1), t.setAlignedSeqs(a, k, E, i, P, G, e.buildAlignMap(a, i)), t);
    }),
    (e.alignDNAprobe = function (t, s) {
      t.param || (t.param = {});
      var n,
        a,
        i,
        u,
        f = [],
        h = [],
        d,
        p,
        v,
        _,
        b,
        w = t.param.gap || -20,
        y = t.param.extend || -1,
        A = [],
        q,
        T = 0,
        N = [0, 0],
        M = !1,
        k = 20,
        E = 2,
        P = -4,
        G = t.param.mat || {
          a: { a: E, c: P, g: P, t: P },
          c: { a: P, c: E, g: P, t: P },
          g: { a: P, c: P, g: E, t: P },
          t: { a: P, c: P, g: P, t: E },
        };
      if (t.fullSeq1 === "" || t.fullSeq2 === "") return null;
      for (
        s
          ? ((n = (t.fullSeq1.toLowerCase() + "-").split("").reverse()), (a = (t.fullSeq2.toLowerCase() + "-").split("").reverse()))
          : ((n = ("-" + t.fullSeq1.toLowerCase()).split("")), (a = ("-" + t.fullSeq2.toLowerCase()).split(""))),
          _ = n.length,
          b = a.length,
          d = 0;
        d < _;
        ++d
      )
        for (f[d] = [], h[d] = [], p = 0; p < b; ++p) ((f[d][p] = 0), (h[d][p] = []));
      for (f[1][0] = 0, h[1][0].push(1), d = 2; d < _; ++d) ((f[d][0] = 0), h[d][0].push(-1));
      for (f[0][1] = 0, h[0][1].push(2), p = 2; p < b; ++p) ((f[0][p] = 0), h[0][p].push(-1));
      for (d = 1; d < _; ++d)
        for (p = 1; p < b; ++p)
          ((A[0] = f[d - 1][p - 1] + G[n[d]][a[p]]),
            h[d - 1][p].indexOf(1) != -1 ? (A[1] = f[d - 1][p] + y) : (A[1] = f[d - 1][p] + w),
            h[d][p - 1].indexOf(2) != -1 ? (A[2] = f[d][p - 1] + y) : (A[2] = f[d][p - 1] + w),
            M ? (q = Math.max(A[0], A[1], A[2])) : (q = Math.max(A[0], A[1], A[2], 0)),
            !M && q > k && (M = !0),
            (f[d][p] = q),
            q === 0 && !M && h[d][p].push(-1),
            q === A[0] && h[d][p].push(0),
            q === A[1] && h[d][p].push(1),
            q === A[2] && h[d][p].push(2));
      for (T = -999, d = n.length - 1, v = 1; v < a.length; ++v) f[d][v] > T && ((T = f[d][v]), (p = v));
      ((N = [d, p]), (i = ""), (u = ""));
      e: for (; h[d][p].length !== 0; )
        switch (h[d][p][h[d][p].length - 1]) {
          case 0:
            ((i = n[d--] + i), (u = a[p--] + u));
            break;
          case 1:
            ((i = n[d--] + i), (u = "-" + u));
            break;
          case 2:
            ((u = a[p--] + u), (i = "-" + i));
            break;
          case -1:
            break e;
        }
      s && ((i = i.split("").reverse().join("")), (u = u.split("").reverse().join("")));
      var O, K, D, L;
      return (
        (O = d),
        (D = p),
        (K = N[0] - 1),
        (L = N[1] - 1),
        s && ((O = n.length - N[0] - 1), (K = n.length - d - 2), (D = a.length - N[1] - 1), (L = a.length - p - 2)),
        t.setAlignedSeqs(i, O, K, u, D, L, e.buildAlignMap(i, u)),
        t
      );
    }),
    (e.alignDNAhalfFit = function (t, s) {
      t.param || (t.param = {});
      var n,
        a,
        i,
        u,
        f = [],
        h = [],
        d,
        p,
        v,
        _,
        b = t.param.gap || -12,
        w = t.param.extend || -1,
        y = [],
        A,
        q = 0,
        T = [0, 0],
        N = 2,
        M = -4,
        k = t.param.mat || {
          a: { a: N, c: M, g: M, t: M },
          c: { a: M, c: N, g: M, t: M },
          g: { a: M, c: M, g: N, t: M },
          t: { a: M, c: M, g: M, t: N },
        };
      if (t.fullSeq1 === "" || t.fullSeq2 === "") return null;
      for (
        s
          ? ((n = (t.fullSeq1.toLowerCase() + "-").split("").reverse()), (a = (t.fullSeq2.toLowerCase() + "-").split("").reverse()))
          : ((n = ("-" + t.fullSeq1.toLowerCase()).split("")), (a = ("-" + t.fullSeq2.toLowerCase()).split(""))),
          v = n.length,
          _ = a.length,
          d = 0;
        d < v;
        ++d
      )
        for (f[d] = [], h[d] = [], p = 0; p < _; ++p) ((f[d][p] = 0), (h[d][p] = []));
      for (f[1][0] = 0, h[1][0].push(1), d = 2; d < v; ++d) ((f[d][0] = d * b), h[d][0].push(1));
      for (f[0][1] = 0, h[0][1].push(2), p = 2; p < _; ++p) ((f[0][p] = 0), h[0][p].push(-1));
      for (d = 1; d < v; ++d)
        for (p = 1; p < _; ++p)
          ((y[0] = f[d - 1][p - 1] + k[n[d]][a[p]]),
            h[d - 1][p].indexOf(1) != -1 ? (y[1] = f[d - 1][p] + w) : (y[1] = f[d - 1][p] + b),
            h[d][p - 1].indexOf(2) != -1 ? (y[2] = f[d][p - 1] + w) : (y[2] = f[d][p - 1] + b),
            (A = Math.max(y[0], y[1], y[2])),
            (f[d][p] = A),
            f[d][p] > q && ((q = f[d][p]), (T = [d, p])),
            A === y[0] && h[d][p].push(0),
            A === y[1] && h[d][p].push(1),
            A === y[2] && h[d][p].push(2));
      ((d = T[0]), (p = T[1]), (i = ""), (u = ""));
      e: for (; h[d][p].length !== 0; )
        switch (h[d][p][h[d][p].length - 1]) {
          case 0:
            ((i = n[d--] + i), (u = a[p--] + u));
            break;
          case 1:
            ((i = n[d--] + i), (u = "-" + u));
            break;
          case 2:
            ((u = a[p--] + u), (i = "-" + i));
            break;
          case -1:
            break e;
        }
      s && ((i = i.split("").reverse().join("")), (u = u.split("").reverse().join("")));
      var E, P, G, O;
      return (
        (E = d),
        (G = p),
        (P = T[0] - 1),
        (O = T[1] - 1),
        s && ((E = n.length - T[0] - 1), (P = n.length - d - 2), (G = a.length - T[1] - 1), (O = a.length - p - 2)),
        t.setAlignedSeqs(i, E, P, u, G, O, e.buildAlignMap(i, u)),
        t
      );
    }),
    (e.buildAlignMap = function (t, s) {
      var n = t.split(""),
        a = s.split(""),
        i,
        u = "",
        f;
      for (f = 0; f < n.length; ++f) n[f] === a[f] ? (u += "|") : n[f] === "-" || a[f] === "-" ? (u += "^") : (u += "x");
      if (((i = u.split("")), i[0] === "x" || i[0] === "^")) {
        for (f = 0; f < i.length - 1; ++f)
          if (i[f] === "x" || i[f] === "^")
            if (i[f + 1] !== "x" && i[f + 1] !== "^") {
              i[f] === "^" && (i[f] = ".");
              break;
            } else i[f] = ".";
      }
      if (i[i.length - 1] === "x" || i[i.length - 1] === "^") {
        for (f = i.length - 1; f > 1; --f)
          if (i[f] === "x" || i[f] === "^")
            if (i[f - 1] !== "x" && i[f - 1] !== "^") {
              i[f] === "^" && (i[f] = ".");
              break;
            } else i[f] = ".";
      }
      u = i.join("");
      for (var h = /x{2,}/g, d = /x*\^+x+|x+\^+x*/g, p, v, _; (p = d.exec(u)); )
        for (v = p.index, _ = p[0].length, f = v; f < v + _; ++f) i[f] = "=";
      for (; (p = h.exec(u)); ) for (v = p.index, _ = p[0].length, f = v; f < v + _; ++f) i[f] = "_";
      return i.join("");
    }),
    (e.Alignment = function (t, s, n, a) {
      ((this.type = t),
        (this.fullSeq1 = this.sanitize(s)),
        (this.fullSeq2 = this.sanitize(n)),
        (this.param = a),
        (this.alnStart1 = 0),
        (this.alnEnd1 = 0),
        (this.alnStart2 = 0),
        (this.alnEnd2 = 0),
        (this.alnSeq1 = ""),
        (this.alnSeq2 = ""),
        (this.alnMap = ""),
        (this.offset = 0));
    }),
    (e.Alignment.prototype.coordS2 = function (t) {
      var s = null,
        n = 0,
        a = 0,
        i = 0;
      if (t < 0 || t >= this.fullSeq1.length)
        throw { name: "coordinate out of bounds ", description: "coordinate " + t + " was out of bounds" };
      if (t < this.alnStart1) return ((s = this.alnStart2 - (this.alnStart1 - t)), s < 0 ? null : s);
      if (t > this.alnEnd1) return ((s = this.alnEnd2 + (t - this.alnEnd1)), s >= this.fullSeq2.length ? null : s);
      for (s = this.alnStart2, n = this.alnStart1, a = this.alnStart2, i = 0; n !== t; )
        (this.alnSeq1[i] !== "-" && this.alnSeq2[i] !== "-" ? (++n, ++a) : this.alnSeq1[i] === "-" ? ++a : ++n, ++i);
      return this.alnSeq2[i] === "-" ? "-" : a;
    }),
    (e.Alignment.prototype.coordS1Offset = function (t) {
      var s = 0,
        n = 0;
      if (t < 0 || t >= this.fullSeq1.length)
        throw { name: "coordinate out of bounds ", description: "coordinate " + t + " was out of bounds" };
      if (t < this.alnStart1 || t > this.alnEnd1) return null;
      for (s = this.alnStart1, n = 0; s !== t; ) this.alnSeq1[n] !== "-" ? (++s, ++n) : this.alnSeq1[n] === "-" && ++n;
      return n;
    }),
    (e.Alignment.prototype.coordS1 = function (t) {
      var s = null,
        n = 0,
        a = 0,
        i = 0;
      if (t < 0 || t >= this.fullSeq2.length)
        throw { name: "coordinate out of bounds ", description: "coordinate " + t + " was out of bounds" };
      if (t < this.alnStart2) return ((s = this.alnStart1 - (this.alnStart2 - t)), s < 0 ? null : s);
      if (t > this.alnEnd2) return ((s = this.alnEnd1 + (t - this.alnEnd2)), s >= this.fullSeq1.length ? null : s);
      for (s = this.alnStart1, n = this.alnStart1, a = this.alnStart2, i = 0; a !== t; )
        (this.alnSeq1[i] !== "-" && this.alnSeq2[i] !== "-" ? (++n, ++a) : this.alnSeq2[i] === "-" ? ++n : ++a, ++i);
      return this.alnSeq1[i] === "-" ? "-" : n;
    }),
    (e.Alignment.prototype.coordS2Offset = function (t) {
      var s = 0,
        n = 0;
      if (t < 0 || t >= this.fullSeq2.length)
        throw { name: "coordinate out of bounds ", description: "coordinate " + t + " was out of bounds" };
      if (t < this.alnStart2 || t > this.alnEnd2) return null;
      for (s = this.alnStart2, n = 0; s !== t; ) this.alnSeq2[n] !== "-" ? (++s, ++n) : this.alnSeq2[n] === "-" && ++n;
      return n;
    }),
    (e.Alignment.prototype.sanitize = function (t) {
      return t.toLowerCase().replace(/[^acgt]+/g, "");
    }),
    (e.Alignment.prototype.setAlignedSeqs = function (t, s, n, a, i, u, f) {
      ((this.offset = i - s),
        (this.alnSeq1 = t),
        (this.alnSeq2 = a),
        (this.alnStart1 = s),
        (this.alnStart2 = i),
        (this.alnEnd1 = n),
        (this.alnEnd2 = u),
        (this.alnMap = f));
    }),
    (e.Alignment.prototype.isAlignedAtS1Coord = function (t) {
      var s = this.alnStart1,
        n = this.alnEnd1;
      return t >= s && t <= n;
    }),
    (e.Alignment.prototype.isAlignedAtS2Coord = function (t) {
      var s = this.alnStart2,
        n = this.alnEnd2;
      return t >= s && t <= n;
    }),
    (e.Alignment.prototype.isS2PresentAtS1Coord = function (t) {
      var s = this.coordS2(t);
      return s === null ? !1 : s === "-" ? !0 : s >= 0 && s <= this.fullSeq2.length - 1;
    }),
    (e.Alignment.prototype.isS1PresentAtS2Coord = function (t) {
      var s = this.coordS1(t);
      return s === null ? !1 : s === "-" ? !0 : s >= 0 && s <= this.fullSeq1.length - 1;
    }),
    (e.Alignment.prototype.getBasesAtS1Coord = function (t) {
      var s = 0,
        n = 0;
      if (this.isAlignedAtS1Coord(t)) for (n = this.alnStart1, s = 0; n !== t; ) (this.alnSeq1[s] !== "-" && ++n, ++s);
      else t < this.alnStart1 ? (s = t - this.alnStart1) : (s = this.alnSeq1.length - 1 + (t - this.alnEnd1));
      return this.getBasesAtOffset(s);
    }),
    (e.Alignment.prototype.getBasesAtS2Coord = function (t) {
      var s = 0,
        n = 0;
      if (this.isAlignedAtS2Coord(t)) for (n = this.alnStart2, s = 0; n !== t; ) (this.alnSeq2[s] !== "-" && ++n, ++s);
      else t < this.alnStart2 ? (s = t - this.alnStart2) : (s = this.alnSeq2.length - 1 + (t - this.alnEnd2));
      return this.getBasesAtOffset(s);
    }),
    (e.Alignment.prototype.getBasesAtOffset = function (t) {
      var s = "",
        n = "";
      return (
        t >= 0 && t < this.alnSeq1.length
          ? ((s = this.alnSeq1[t]), (n = this.alnSeq2[t]))
          : t < 0
            ? (this.alnStart1 + t >= 0 ? (s = this.fullSeq1[this.alnStart1 + t]) : (s = ""),
              this.alnStart2 + t >= 0 ? (n = this.fullSeq2[this.alnStart2 + t]) : (n = ""))
            : (this.alnEnd1 + (t - this.alnSeq1.length + 1) < this.fullSeq1.length
                ? (s = this.fullSeq1[this.alnEnd1 + (t - this.alnSeq1.length + 1)])
                : (s = ""),
              this.alnEnd2 + (t - this.alnSeq2.length + 1) < this.fullSeq2.length
                ? (n = this.fullSeq2[this.alnEnd2 + (t - this.alnSeq2.length + 1)])
                : (n = "")),
        [s, n]
      );
    }),
    e
  );
})(po);
const Li = po,
  Ot = {};
Ot.CODON_TARGETS = "ACDEFGHIKLMNPQRSTVWY*X";
Ot.codonOpRegex = /^([A-Z*])(\d+)([A-Z*])(?::([A-Z]+))?$/;
Ot.codonOpError = function (l) {
  var e = Ot.codonOpRegex.exec(l.toUpperCase());
  if (!e) return "skipping op [" + l + "] - unable to parse (expected e.g. M2A or M2G:GGG)";
  if (Number(e[2]) < 1) return "skipping op [" + l + "] - codon numbers start at 1";
  if (Ot.CODON_TARGETS.indexOf(e[3]) === -1) return "skipping op [" + l + "] - no codons available for amino acid " + e[3];
  if (e[4] !== void 0 && !/^[ACGT]{3}$/.test(e[4]))
    return "skipping op [" + l + "] - the triplet after the colon must be 3 bases (A, C, G or T)";
  if (e[4] !== void 0 && /^[ACGT]{3}$/.test(e[4]) && e[3] !== "X") {
    var t = Te.translate(e[4]);
    if (t !== e[3]) return "skipping op [" + l + "] - triplet " + e[4] + " encodes " + t + ", not " + e[3];
  }
  return "";
};
Ot.codonRangeError = function (l, e, t, s, n) {
  var a = t - 1 + (e - 1) * 3,
    i = n ? e * 3 > s : a + 3 > s;
  return i ? "skipping op [" + l + "] - codon " + e + " is beyond the end of the sequence" : "";
};
Ot.checkCodonOp = function (l, e, t, s) {
  var n = Ot.codonOpError(l);
  if (n) return n;
  if (!(e >= 1)) return "skipping op [" + l + "] - no valid ORF start";
  var a = Number(Ot.codonOpRegex.exec(l.toUpperCase())[2]);
  return Ot.codonRangeError(l, a, e, t, s);
};
Ot.VARIANT_COUNTS = { A: 1, C: 1, G: 1, T: 1, R: 2, Y: 2, K: 2, M: 2, S: 2, W: 2, B: 3, D: 3, H: 3, V: 3, N: 4 };
Ot.MAX_VARIANTS = 256;
Ot.stripReplacement = function (l) {
  return (l || "").replace(/\s+/g, "");
};
Ot.replacementError = function (l, e) {
  var t = Ot.stripReplacement(l),
    s = t.replace(/[ACGTRYKMSWBDHVN]/gi, "");
  if (s) return "replacement sequence contains invalid characters: " + Array.from(new Set(s)).join(" ");
  if (e !== !0) {
    for (var n = 1, a = 0; a < t.length; a++) n *= Ot.VARIANT_COUNTS[t[a].toUpperCase()] || 1;
    if (n > Ot.MAX_VARIANTS) return "too many variants (" + n + "); use at most " + Ot.MAX_VARIANTS;
  }
  return "";
};
Ot.batchLineError = function (l) {
  return l && l.length > 4 ? "replacement contains spaces" : "";
};
Ot.positionError = function (l, e, t) {
  var s = Number(e);
  if (!Number.isInteger(s) || s < 1 || s > t) {
    var n = e == null || e === "" ? "empty" : e;
    return l + " must be a whole number from 1 to " + t + " (got " + n + ")";
  }
  return "";
};
Ot.MAX_SEQ_LEN = 5e4;
Ot.sequenceLengthError = function (l) {
  return l > Ot.MAX_SEQ_LEN ? "sequence length (" + l + " bp) exceeds the maximum of " + Ot.MAX_SEQ_LEN + " bp" : "";
};
Ot.normalizeOpText = function (l) {
  return (l || "")
    .trim()
    .replace(/[\s,]+/g, " ")
    .toUpperCase();
};
Ot.duplicateOpsWarning = function (l) {
  var e = new Set();
  return (l || []).map(function (t) {
    var s = Ot.normalizeOpText(t);
    return e.has(s) ? "duplicate of an earlier op" : (e.add(s), "");
  });
};
Ot.escapeRegExp = function (l) {
  return l.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
};
const jt = Ot,
  vt = {};
vt.MIN_FLANK_5 = 35;
vt.MIN_FLANK_3 = 25;
vt.flankError = function (l, e, t, s, n, a) {
  return s
    ? ""
    : ((n = n === void 0 ? vt.MIN_FLANK_5 : n),
      (a = a === void 0 ? vt.MIN_FLANK_3 : a),
      l < n
        ? "insufficient 5' flanking sequence to build primers for partial sequence (need at least " +
          n +
          " bp before the edit, have " +
          Math.max(l, 0) +
          ")"
        : t + 1 - e < a
          ? "insufficient 3' flanking sequence to build primers for partial sequence (need at least " +
            a +
            " bp after the edit, have " +
            Math.max(t + 1 - e, 0) +
            ")"
          : "");
};
vt.notIntegerError = function (l, e) {
  if (!Number.isInteger(l) || !Number.isInteger(e)) {
    var t = function (s) {
      return s == null || s === "" ? "empty" : s;
    };
    return "positions must be whole numbers (got " + t(l) + ", " + t(e) + ")";
  }
  return "";
};
vt.normalizeFlanks = function (l, e, t, s) {
  var n = vt.notIntegerError(l, e);
  if (n) return { start: l, end: e, error: n };
  if (s) {
    var a = function (u) {
      return ((((u - 1) % t) + t) % t) + 1;
    };
    return { start: a(l), end: a(e), error: "" };
  }
  var i = "";
  return (
    l < 0
      ? (i = "the edit starts before the start of the sequence")
      : e > t + 1
        ? (i = "the edit runs beyond the end of the sequence")
        : l >= e && (i = "the end position must be after the start on a partial (linear) sequence"),
    { start: l, end: e, error: i }
  );
};
vt.canonicalFlanks = function (l, e, t, s) {
  if (!s) return { start: l, end: e };
  var n = vt.normalizeFlanks(l, e, t, !0),
    a = n.start;
  return (a === t && n.end > 1 && (a = 0), { start: a, end: n.end });
};
vt.codonAt = function (l, e, t) {
  if (!t) return l.slice(e, e + 3);
  for (var s = l.length, n = "", a = 0; a < 3; a++) n += l.charAt((((e + a) % s) + s) % s);
  return n;
};
vt.applyMutation = function (l, e, t, s, n) {
  s = s || "";
  var a = vt.canonicalFlanks(e, t, l.length, n);
  return ((e = a.start), (t = a.end), !n || t > e ? l.slice(0, e) + s + l.slice(t - 1) : l.slice(t - 1, e) + s);
};
vt.classifyOp = function (l, e, t, s, n) {
  t = t || "";
  var a = n ? (e - l + s) % s : e - l,
    i = { op: "", error: "", warning: "" };
  if (a === 0) return ((i.error = "insertions need consecutive positions (e.g. 200 201)"), i);
  if ((a === 1 ? (i.op = "insert") : t.length === 0 ? (i.op = "delete") : (i.op = "replace"), n)) {
    var u = l === 0 ? s : l,
      f = e === 0 ? s : e;
    if (f < u) {
      var h = a - 1;
      if (h > s / 2) {
        var d = s - h;
        i.warning = "this edit spans the origin and removes " + h + " bp, leaving " + d + " bp - check that start and end are not swapped";
      }
    }
  }
  return i;
};
vt.mergeCodonOps = function (l, e, t) {
  var s = l.length,
    n = function (P) {
      return { start: 0, end: 0, repseq: "", error: P };
    };
  if (!e || e.length === 0) return n("no mutations to merge");
  for (var a = 0; a < e.length; a++)
    for (var i = a + 1; i < e.length; i++) {
      var u = e[a],
        f = e[i];
      if (u.start < f.end - 1 && f.start < u.end - 1) {
        var h = ((u.id || "") + " " + (f.id || "")).trim(),
          d = u.start === f.start && u.end === f.end;
        return n(h + ": " + (d ? "duplicate mutation at the same codon" : "overlapping codon positions"));
      }
    }
  var p = e.slice().sort(function (P, G) {
      return P.start - G.start;
    }),
    v = p[0],
    _ = p[p.length - 1],
    b = _.end - v.start,
    w = p
      .map(function (P) {
        return P.id || "";
      })
      .join(" ")
      .trim();
  if (b > p.length * 8) return n(w + ": mutations are too widely spread to safely merge (" + b + " bp)");
  if (b > 60) return n(w + ": merged span of " + b + " bp exceeds the 60 bp limit for a safe merge");
  for (
    var y = function (P) {
        return t ? l.charAt((((P - 1) % s) + s) % s) : l.charAt(P - 1);
      },
      A = p[0].codon,
      q = p[0].end,
      T = 1;
    T < p.length;
    T++
  ) {
    for (var N = p[T], M = q; M <= N.start; M++) A += y(M);
    ((A += N.codon), (q = N.end));
  }
  var k = v.start,
    E = _.end;
  return (t && ((k = ((k % s) + s) % s), (E = ((E % s) + s) % s)), { start: k, end: E, repseq: A, error: "" });
};
vt.matchesIUPAC = function (l, e) {
  if (!l || !e || l.length !== e.length) return !1;
  for (var t = 0; t < l.length; t++) {
    var s = Te.DEGENERATES[e.charAt(t).toUpperCase()],
      n = Te.DEGENERATES[l.charAt(t).toUpperCase()];
    if (!s || !n) return !1;
    for (var a = 0; a < n.length; a++) if (s.indexOf(n.charAt(a)) === -1) return !1;
  }
  return !0;
};
var Gi = function (l, e, t) {
  if (t) return (l || "").toUpperCase() === t.toUpperCase();
  if (((e = (e || "").toUpperCase()), e === "X")) return vt.matchesIUPAC(l, "NNK");
  if (!l || l.length !== 3) return !1;
  var s;
  try {
    s = Te.translate(l);
  } catch {
    return !1;
  }
  return s === e;
};
vt.verifyOp = function (l, e, t, s, n, a) {
  if (((e = e || ""), !l)) return "";
  var i = t.length,
    u = function (E) {
      return E ? E + ": " : "";
    };
  if (l.kind === "codon") {
    var f = a ? (((n - s) % i) + i) % i : n - s;
    if (f - 1 !== 3) return "the op's flanks bracket " + (f - 1) + " bp, not one codon (3 bp)";
    if (!Gi(e, l.target, l.triplet)) {
      if (l.triplet) return "repseq " + e + " does not match the explicit triplet " + l.triplet;
      if ((l.target || "").toUpperCase() === "X") return "target X must be within the degenerate NNK codon, got " + e;
      var h;
      try {
        h = Te.translate(e);
      } catch {
        h = "?";
      }
      return "repseq " + e + " translates to " + h + ", not the target " + l.target;
    }
    return "";
  }
  if (l.kind === "merge") {
    var d = l.subs || [];
    if (d.length === 0) return "";
    var p = d[0].start,
      v = d[d.length - 1].start - p + 3;
    if (e.length !== v) return "merged repseq length " + e.length + " does not match the merged span (" + v + " bp)";
    for (
      var _ = function (E) {
          return a ? t.charAt((((E - 1) % i) + i) % i) : t.charAt(E - 1);
        },
        b = p,
        w = 0;
      w < d.length;
      w++
    ) {
      var y = d[w],
        A = y.start - p;
      if (w > 0) {
        for (var q = y.start - b + 1, T = "", N = 0; N < q; N++) T += _(b + N);
        var M = e.slice(A - q, A);
        if (M.toLowerCase() !== T.toLowerCase()) return u(y.id) + "the wild-type bases before this codon do not match the sequence";
      }
      var k = e.slice(A, A + 3);
      if (!Gi(k, y.target, y.triplet)) return u(y.id) + "codon " + k + " at its merged position does not match its target";
      b = y.end;
    }
    return "";
  }
  return "";
};
vt.verifyPair = function (l, e, t) {
  if (!l) return "missing primer pair";
  var s = l.repseq || "",
    n = l.seqf || "",
    a = l.seqr || "",
    i;
  try {
    i = vt.applyMutation(e, l.selstart, l.selend, s, t);
  } catch (E) {
    return "could not reconstruct the mutant sequence (" + (E && E.message ? E.message : E) + ")";
  }
  var u;
  try {
    u = Te.revcomp(a);
  } catch {
    return "reverse primer is not a valid IUPAC sequence";
  }
  var f = (u + n).toLowerCase();
  if (!f) return "empty primer pair";
  var h = vt.canonicalFlanks(l.selstart, l.selend, e.length, t),
    d = h.start,
    p = h.end,
    v = !!t && p <= d,
    _,
    b,
    w;
  if (t) {
    var y = v ? i.length - s.length : d;
    y < 0 && (y = 0);
    var A = i.slice(y) + i.slice(0, y);
    ((_ = (A + A).toLowerCase()), (b = A.length - 1), (w = A.length + s.length));
  } else ((_ = i.toLowerCase()), (b = Math.max(d - 1, 0)), (w = Math.min(d + s.length, i.length - 1)));
  for (var q = !1, T = _.indexOf(f); T !== -1; T = _.indexOf(f, T + 1))
    if (T <= b && T + f.length - 1 >= w) {
      q = !0;
      break;
    }
  if (!q) return "forward/reverse primers do not match the mutant sequence at the edit site";
  var N = function (E, P, G, O) {
      return E === void 0 || P === void 0
        ? ""
        : (E + P).toLowerCase() !== (G || "").toLowerCase()
          ? O + " flap + annealing region does not reconstruct the full primer"
          : "";
    },
    M = N(l.seqfpflap, l.seqfanneal, n, "forward") || N(l.seqrpflap, l.seqranneal, a, "reverse");
  if (M) return M;
  var k = vt.verifyOp(l.op, s, e, l.selstart, l.selend, t);
  return k || "";
};
vt.BUILD_CLEAR_KEYS = ["inputmode", "codonops", "rawbatchinput", "seq"];
vt.buildAction = function (l, e) {
  if (!l || l.tick !== e.tick) return "build";
  for (let t = 0; t < vt.BUILD_CLEAR_KEYS.length; ++t) {
    let s = vt.BUILD_CLEAR_KEYS[t];
    if (l[s] !== e[s]) return "clear";
  }
  return "build";
};
vt.nebuilderBlocker = function (l, e, t) {
  if (l !== "codon") return "NEBuilder primers are only available in codon mode";
  if (t) return 'NEBuilder primers need "Confine mutations to primer 5\u2032 tails" off';
  var s = String(e || "")
    .toUpperCase()
    .trim()
    .split(/[\s,;]+/)
    .filter(Boolean);
  return s.length < 2
    ? "NEBuilder primers need at least two mutations"
    : s.some(function (n) {
          return /^[A-Z]\d+X/.test(n);
        })
      ? "NEBuilder primers can't be used with degenerate (X) mutations"
      : "";
};
vt.NB_MIN_FRAGMENT = 60;
vt.nebuilderFragments = function (l, e, t) {
  var s = [];
  if (!l || l.length === 0) return { primers: [], warnings: s };
  for (
    var n = function (A) {
        return ((Number(A.locf) % e) + e) % e;
      },
      a = l.slice().sort(function (A, q) {
        return n(A) - n(q);
      }),
      i = [],
      u = 0;
    u < a.length;
    ++u
  ) {
    var f = i[i.length - 1];
    if (f && n(f) === n(a[u])) {
      s.push("NEBuilder: " + a[u].idprefix + " starts at the same position as " + f.idprefix + ", so it was left out of the assembly.");
      continue;
    }
    i.push(a[u]);
  }
  for (var h = Te.revcomp, d = i.length, p = [], v = 0; v < d; ++v) {
    var _ = i[v],
      b = {
        seqf: _.seqf,
        lenf: _.seqf.length,
        tmf: _.tmf,
        seqfpflap: _.seqfpflap,
        seqfanneal: _.seqfanneal,
        locf: n(_),
        pairIndex: v + 1,
        nebuilder: !0,
      };
    if (t === "multiple" && d > 1) {
      var w = i[(v + 1) % d],
        y = (((n(w) - n(_)) % e) + e) % e;
      Object.assign(b, {
        seqr: h(w.seqf),
        tmr: w.tmf,
        seqrpflap: h(w.seqfpflap),
        seqranneal: w.seqrnbanneal,
        idprefix: _.idprefix + "_" + w.idprefix,
        description: _.description + " " + w.description,
        locr: (n(w) + w.seqrnbanneal.length) % e,
        wraparound: n(w) < n(_),
        clash: y < vt.NB_MIN_FRAGMENT,
        fraglen: _.seqfpflap.length + y + w.seqrnbanneal.length + h(w.seqfpflap).length,
      });
    } else
      Object.assign(b, {
        seqr: h(_.seqf),
        tmr: _.tmf,
        seqrpflap: h(_.seqfpflap),
        seqranneal: _.seqrnbanneal,
        idprefix: _.idprefix,
        description: _.description,
        locr: (n(_) + _.seqrnbanneal.length) % e,
        fraglen: e + _.seqrnbanneal.length,
      });
    ((b.lenr = b.seqr.length), (b.ta = Math.min(Math.min(b.tmf, b.tmr) + 1, 72)), p.push(b));
  }
  return { primers: p, warnings: s };
};
vt.ORF_MIN_CODONS = 10;
vt.orfStarts = function (l, e) {
  e = e || vt.ORF_MIN_CODONS;
  for (var t = String(l || "").toUpperCase(), s = [], n = /[ACGT]+/g, a; (a = n.exec(t)) !== null; ) {
    for (var i = a[0], u = [-1, -1, -1], f = 0; f + 3 <= i.length; ++f) {
      var h = i.substr(f, 3);
      (h === "TAA" || h === "TAG" || h === "TGA") && (u[f % 3] = f);
    }
    for (var d = 0; d + 3 <= i.length; ++d) i.substr(d, 3) === "ATG" && u[d % 3] >= d + 3 * e && s.push(a.index + d);
  }
  return s;
};
const ct = vt,
  Dt = {};
Dt.finalSeqRecord = function (l, e) {
  var t = function (w) {
    return { fasta: "", error: w };
  };
  if (!e || !e.seq) return t("no sequence is loaded");
  if (!l) return t("no primer pair is selected");
  var s = e.seq,
    n = !!e.circular,
    a = l.selstart,
    i = l.selend,
    u = l.seqop,
    f = l.repseq || "",
    h = l.idprefix || "resultseq";
  l.pairIndex && (h += "_" + l.pairIndex);
  var d = String(l.description || u || "").trim();
  if (a == null || i == null) return t("the pair has no start/end positions");
  var p = ct.normalizeFlanks(a, i, s.length, n).error;
  if (p) return t(p);
  var v = ">" + e.id + "_" + h + (d ? " " + d : ""),
    _ = ct.canonicalFlanks(a, i, s.length, n);
  n && _.end <= _.start && (v += "; rotated to start at base " + _.end);
  var b = ct.applyMutation(s, _.start, _.end, f, n).toLowerCase();
  return {
    fasta:
      v +
      `
` +
      Dt.wrap(b, Dt.FASTA_WIDTH),
    error: "",
  };
};
Dt.buildFasta = function (l, e) {
  var t = [],
    s = [];
  if (!l || l.length === 0) return { data: "", warnings: ["Sequence download: there are no primer pairs to build a sequence from."] };
  for (var n = 0; n < l.length; ++n) {
    var a = l[n],
      i = a && a.idprefix ? a.idprefix : "pair " + (n + 1),
      u;
    try {
      u = Dt.finalSeqRecord(a, e);
    } catch (f) {
      u = { fasta: "", error: "unexpected error (" + (f && f.message ? f.message : f) + ")" };
    }
    u.error ? s.push("Sequence download: could not build the sequence for " + i + ": " + u.error + ".") : t.push(u.fasta);
  }
  return { data: t.join(""), warnings: s };
};
Dt.FASTA_WIDTH = 70;
Dt.wrap = function (l, e) {
  for (var t = "", s = 0; s < l.length; s += e)
    t +=
      l.slice(s, s + e) +
      `
`;
  return t;
};
Dt.primerId = function (l, e) {
  return l.idprefix + "_" + l.pairIndex + "_" + (e === "fwd" ? "F" : "R");
};
Dt.primerSeq = function (l, e) {
  var t = function (s) {
    return String(s || "").toLowerCase();
  };
  return e === "fwd"
    ? l.seqfanneal == null
      ? l.seqf || ""
      : t(l.seqfpflap) + l.seqfanneal
    : l.seqranneal == null
      ? l.seqr || ""
      : l.nebuilder
        ? l.seqranneal + t(l.seqrpflap)
        : t(l.seqrpflap) + l.seqranneal;
};
Dt.tsvCell = function (l) {
  var e = String(l == null ? "" : l).replace(/[\t\r\n]+/g, " ");
  return /^[=+\-@]/.test(e) ? "'" + e : e;
};
Dt.downloadName = function (l, e, t) {
  var s = String(l == null ? "" : l).trim();
  return (s || (s = e), s.endsWith(t) || (s += t), s);
};
Dt.shorten = function (l) {
  return ((l = l || ""), l.length > 25 ? l.slice(0, 10) + "..." + l.slice(-10) : l);
};
Dt.describeOp = function (l) {
  var e = l.seq || "",
    t = e.length,
    s = !!l.circular,
    n = s ? ct.canonicalFlanks(l.start, l.end, t, !0) : { start: l.start, end: l.end },
    a = function (_, b) {
      return _ === b ? "at base " + _ : "at bases " + _ + "-" + b;
    },
    i = " in " + l.seqName;
  if (l.seqop === "insert") {
    var u = s && n.start === 0 ? t : n.start,
      f = n.end;
    return !s && u < 1
      ? "Insert " + l.newseq + " before base 1" + i
      : !s && f > t
        ? "Insert " + l.newseq + " after base " + t + i
        : "Insert " + l.newseq + " between bases " + u + "-" + f + i;
  }
  var h = s && n.end <= n.start,
    d = (h ? e.slice(n.start) + e.slice(0, n.end - 1) : e.slice(n.start, n.end - 1)).toLowerCase(),
    p = (n.start % t) + 1,
    v = n.end - 1 < 1 ? t : n.end - 1;
  return l.seqop === "delete"
    ? "Delete " + Dt.shorten(d) + " " + a(p, v) + i
    : "Replace " + Dt.shorten(d) + " " + a(p, v) + " with " + l.newseq + i;
};
Dt.primerList = function (l) {
  for (var e = "", t = 0; t < l.length; ++t) {
    var s = l[t],
      n = "Ta=" + Math.round(s.ta);
    ((e +=
      Dt.tsvCell(Dt.primerId(s, "fwd")) +
      "	" +
      Dt.primerSeq(s, "fwd") +
      "	Tm=" +
      Math.round(s.tmf) +
      " " +
      n +
      `
`),
      (e +=
        Dt.tsvCell(Dt.primerId(s, "rev")) +
        "	" +
        Dt.primerSeq(s, "rev") +
        "	Tm=" +
        Math.round(s.tmr) +
        " " +
        n +
        `
`));
  }
  return e;
};
Dt.summary = function (l) {
  for (
    var e = [
        [
          "ID",
          "Description",
          "Fwd Primer ID",
          "Fwd Primer Seq",
          "Length",
          "Tm Fwd",
          "Rev Primer ID",
          "Rev Primer Seq",
          "Length",
          "Tm Rev",
          "Ta",
        ],
      ],
      t = 0;
    t < l.length;
    ++t
  ) {
    var s = l[t];
    e.push([
      s.idprefix,
      String(s.description || "").trim(),
      Dt.primerId(s, "fwd"),
      Dt.primerSeq(s, "fwd"),
      s.lenf,
      Math.round(s.tmf),
      Dt.primerId(s, "rev"),
      Dt.primerSeq(s, "rev"),
      s.lenr,
      Math.round(s.tmr),
      Math.round(s.ta),
    ]);
  }
  return (
    e.map(function (n) {
      return n.map(Dt.tsvCell).join("	");
    }).join(`
`) +
    `
`
  );
};
const Jt = Dt,
  ho = Si({ sdm: null, seqinput: null }),
  mo = (l, e) => ho.update((t) => ({ ...t, [l]: e })),
  go = (l) => To(ho)[l] || null,
  ni = {};
ni.revcompSeq = function (l) {
  if (!l) return null;
  var e = Te.revcomp(l, !0);
  return e;
};
ni.parseSeqFile = function (l, e, t) {
  var s = l;
  function n(v) {
    v = v.toLowerCase();
    var _ = "",
      b = /\.(\w+)$/.exec(v);
    return (b && b[1] && (_ = b[1]), _);
  }
  function a(v) {
    return v.search(/^[\s\r\n]*>/) !== -1 ? u(v) : v.search(/^[\s\r\n]*LOCUS/) !== -1 ? f(v) : i(v);
  }
  function i(v) {
    var _ = Te.sanitizeSeq(v),
      b = { id: "seq", seq: _, isCircular: !1, type: "dna", topology: "linear", format: "raw", recordCount: 1, length: _.length };
    return b;
  }
  function u(v) {
    var _, b, w, y, A;
    y = v.split(/[\r\n]+|\r|\n/);
    for (var q = 0; q < y.length && y[q].trim() == ""; ) q++;
    ((w = /^\s*\>(\S+)\s*(.*)$/.exec(y[q])),
      w && w.length > 0 ? ((_ = w[1] || "seq"), _[_.length - 1] === " " && (_ = _.splice(0, _.length - 1))) : (_ = "seq"));
    for (var T = 0, N = q; N < y.length; N++) /^\s*>/.test(y[N]) && T++;
    (T === 0 && (T = 1), (y = y.slice(q + 1)));
    var M = y.findIndex(function (k) {
      return /^\s*>/.test(k);
    });
    return (
      M !== -1 && (y = y.slice(0, M)),
      (b = Te.sanitizeSeq(y.join(""))),
      (A = { id: _, seq: b, isCircular: !1, type: "dna", topology: "linear", format: "fasta", recordCount: T, length: b.length }),
      A
    );
  }
  function f(v) {
    var _ = v.split(/\r\n|\r|\n/),
      b = _.filter(function (L) {
        return /^LOCUS\s/.test(L);
      }).length;
    b === 0 && (b = 1);
    for (var w = 0; w < _.length && _[w].trim() === ""; ) ++w;
    var y = _[w] || "",
      A = /^LOCUS\s+(\S+)\s/,
      q = A.exec(y),
      T = (q && q[1]) || "seq",
      N = !1,
      M = "unknown",
      k = q ? y.slice(q[0].length) : y,
      E = /\b(linear|circular)\b/i.exec(k);
    E && ((M = E[1].toLowerCase()), (N = M === "circular"));
    for (var P = 0; P < _.length && !_[P].startsWith("ORIGIN"); ) ++P;
    if (P === _.length) throw new Error("GenBank entry has no ORIGIN section");
    (++P, _.splice(0, P));
    for (var G = /^\/\//, O = 0; O < _.length && !G.test(_[O]); ) ++O;
    var K = _.slice(0, O).join("");
    K = Te.sanitizeSeq(K);
    var D = { id: T, seq: K, isCircular: N, type: "dna", topology: M, format: "genbank", recordCount: b, length: K.length };
    return D;
  }
  var h = { any: a, raw: i, fasta: u, genbank: f };
  ((t = t || "any"), (t = t.toLowerCase()));
  var d;
  if (!t && e) {
    var p = n(e);
    switch (p) {
      case "fa":
      case "fasta":
      case "fas":
      case "fna":
      case "ffn":
        t = "fasta";
        break;
      case "gb":
      case "gbk":
        t = "genbank";
        break;
      default:
        t = "any";
    }
  } else t = "any";
  if (t in h) d = h[t](s);
  else throw new Error("parseSeqFile: invalid sequence format type");
  return d;
};
ni.test = "seqinput lib";
const Xo = ni;
var An = window.CustomEvent;
(!An || typeof An == "object") &&
  ((An = function (e, t) {
    t = t || {};
    var s = document.createEvent("CustomEvent");
    return (s.initCustomEvent(e, !!t.bubbles, !!t.cancelable, t.detail || null), s);
  }),
  (An.prototype = window.Event.prototype));
function vo(l, e) {
  var t = "on" + e.type.toLowerCase();
  return (typeof l[t] == "function" && l[t](e), l.dispatchEvent(e));
}
function Qo(l) {
  for (; l && l !== document.body; ) {
    var e = window.getComputedStyle(l),
      t = function (s, n) {
        return !(e[s] === void 0 || e[s] === n);
      };
    if (
      e.opacity < 1 ||
      t("zIndex", "auto") ||
      t("transform", "none") ||
      t("mixBlendMode", "normal") ||
      t("filter", "none") ||
      t("perspective", "none") ||
      e.isolation === "isolate" ||
      e.position === "fixed" ||
      e.webkitOverflowScrolling === "touch"
    )
      return !0;
    l = l.parentElement;
  }
  return !1;
}
function Rn(l) {
  for (; l; ) {
    if (l.localName === "dialog") return l;
    l.parentElement ? (l = l.parentElement) : l.parentNode ? (l = l.parentNode.host) : (l = null);
  }
  return null;
}
function _o(l) {
  for (; l && l.shadowRoot && l.shadowRoot.activeElement; ) l = l.shadowRoot.activeElement;
  l && l.blur && l !== document.body && l.blur();
}
function xo(l, e) {
  for (var t = 0; t < l.length; ++t) if (l[t] === e) return !0;
  return !1;
}
function _i(l) {
  return !l || !l.hasAttribute("method") ? !1 : l.getAttribute("method").toLowerCase() === "dialog";
}
function bo(l) {
  var e = ["button", "input", "keygen", "select", "textarea"],
    t = e.map(function (i) {
      return i + ":not([disabled])";
    });
  t.push('[tabindex]:not([disabled]):not([tabindex=""])');
  var s = l.querySelector(t.join(", "));
  if (!s && "attachShadow" in Element.prototype)
    for (var n = l.querySelectorAll("*"), a = 0; a < n.length && !(n[a].tagName && n[a].shadowRoot && ((s = bo(n[a].shadowRoot)), s)); a++);
  return s;
}
function Fi(l) {
  return l.isConnected || document.body.contains(l);
}
function qo(l) {
  if (l.submitter) return l.submitter;
  var e = l.target;
  if (!(e instanceof HTMLFormElement)) return null;
  var t = kt.formSubmitter;
  if (!t) {
    var s = l.target,
      n = ("getRootNode" in s && s.getRootNode()) || document;
    t = n.activeElement;
  }
  return !t || t.form !== e ? null : t;
}
function Jo(l) {
  if (!l.defaultPrevented) {
    var e = l.target,
      t = kt.imagemapUseValue,
      s = qo(l);
    t === null && s && (t = s.value);
    var n = Rn(e);
    if (!!n) {
      var a = (s && s.getAttribute("formmethod")) || e.getAttribute("method");
      a === "dialog" && (l.preventDefault(), t != null ? n.close(t) : n.close());
    }
  }
}
function wo(l) {
  if (
    ((this.dialog_ = l),
    (this.replacedStyleTop_ = !1),
    (this.openAsModal_ = !1),
    l.hasAttribute("role") || l.setAttribute("role", "dialog"),
    (l.show = this.show.bind(this)),
    (l.showModal = this.showModal.bind(this)),
    (l.close = this.close.bind(this)),
    l.addEventListener("submit", Jo, !1),
    "returnValue" in l || (l.returnValue = ""),
    "MutationObserver" in window)
  ) {
    var e = new MutationObserver(this.maybeHideModal.bind(this));
    e.observe(l, { attributes: !0, attributeFilter: ["open"] });
  } else {
    var t = !1,
      s = function () {
        (t ? this.downgradeModal() : this.maybeHideModal(), (t = !1));
      }.bind(this),
      n,
      a = function (i) {
        if (i.target === l) {
          var u = "DOMNodeRemoved";
          ((t |= i.type.substr(0, u.length) === u), window.clearTimeout(n), (n = window.setTimeout(s, 0)));
        }
      };
    ["DOMAttrModified", "DOMNodeRemoved", "DOMNodeRemovedFromDocument"].forEach(function (i) {
      l.addEventListener(i, a);
    });
  }
  (Object.defineProperty(l, "open", { set: this.setOpen.bind(this), get: l.hasAttribute.bind(l, "open") }),
    (this.backdrop_ = document.createElement("div")),
    (this.backdrop_.className = "backdrop"),
    this.backdrop_.addEventListener("mouseup", this.backdropMouseEvent_.bind(this)),
    this.backdrop_.addEventListener("mousedown", this.backdropMouseEvent_.bind(this)),
    this.backdrop_.addEventListener("click", this.backdropMouseEvent_.bind(this)));
}
wo.prototype = {
  get dialog() {
    return this.dialog_;
  },
  maybeHideModal: function () {
    (this.dialog_.hasAttribute("open") && Fi(this.dialog_)) || this.downgradeModal();
  },
  downgradeModal: function () {
    !this.openAsModal_ ||
      ((this.openAsModal_ = !1),
      (this.dialog_.style.zIndex = ""),
      this.replacedStyleTop_ && ((this.dialog_.style.top = ""), (this.replacedStyleTop_ = !1)),
      this.backdrop_.parentNode && this.backdrop_.parentNode.removeChild(this.backdrop_),
      kt.dm.removeDialog(this));
  },
  setOpen: function (l) {
    l
      ? this.dialog_.hasAttribute("open") || this.dialog_.setAttribute("open", "")
      : (this.dialog_.removeAttribute("open"), this.maybeHideModal());
  },
  backdropMouseEvent_: function (l) {
    if (this.dialog_.hasAttribute("tabindex")) this.dialog_.focus();
    else {
      var e = document.createElement("div");
      (this.dialog_.insertBefore(e, this.dialog_.firstChild), (e.tabIndex = -1), e.focus(), this.dialog_.removeChild(e));
    }
    var t = document.createEvent("MouseEvents");
    (t.initMouseEvent(
      l.type,
      l.bubbles,
      l.cancelable,
      window,
      l.detail,
      l.screenX,
      l.screenY,
      l.clientX,
      l.clientY,
      l.ctrlKey,
      l.altKey,
      l.shiftKey,
      l.metaKey,
      l.button,
      l.relatedTarget,
    ),
      this.dialog_.dispatchEvent(t),
      l.stopPropagation());
  },
  focus_: function () {
    var l = this.dialog_.querySelector("[autofocus]:not([disabled])");
    (!l && this.dialog_.tabIndex >= 0 && (l = this.dialog_), l || (l = bo(this.dialog_)), _o(document.activeElement), l && l.focus());
  },
  updateZIndex: function (l, e) {
    if (l < e) throw new Error("dialogZ should never be < backdropZ");
    ((this.dialog_.style.zIndex = l), (this.backdrop_.style.zIndex = e));
  },
  show: function () {
    this.dialog_.open || (this.setOpen(!0), this.focus_());
  },
  showModal: function () {
    if (this.dialog_.hasAttribute("open"))
      throw new Error("Failed to execute 'showModal' on dialog: The element is already open, and therefore cannot be opened modally.");
    if (!Fi(this.dialog_)) throw new Error("Failed to execute 'showModal' on dialog: The element is not in a Document.");
    if (!kt.dm.pushDialog(this)) throw new Error("Failed to execute 'showModal' on dialog: There are too many open modal dialogs.");
    (Qo(this.dialog_.parentElement),
      this.setOpen(!0),
      (this.openAsModal_ = !0),
      kt.needsCentering(this.dialog_) ? (kt.reposition(this.dialog_), (this.replacedStyleTop_ = !0)) : (this.replacedStyleTop_ = !1),
      this.dialog_.parentNode.insertBefore(this.backdrop_, this.dialog_.nextSibling),
      this.focus_());
  },
  close: function (l) {
    if (!this.dialog_.hasAttribute("open"))
      throw new Error(
        "Failed to execute 'close' on dialog: The element does not have an 'open' attribute, and therefore cannot be closed.",
      );
    (this.setOpen(!1), l !== void 0 && (this.dialog_.returnValue = l));
    var e = new An("close", { bubbles: !1, cancelable: !1 });
    vo(this.dialog_, e);
  },
};
var kt = {};
kt.reposition = function (l) {
  var e = document.body.scrollTop || document.documentElement.scrollTop,
    t = e + (window.innerHeight - l.offsetHeight) / 2;
  l.style.top = Math.max(e, t) + "px";
};
kt.isInlinePositionSetByStylesheet = function (l) {
  for (var e = 0; e < document.styleSheets.length; ++e) {
    var t = document.styleSheets[e],
      s = null;
    try {
      s = t.cssRules;
    } catch {}
    if (!!s)
      for (var n = 0; n < s.length; ++n) {
        var a = s[n],
          i = null;
        try {
          i = document.querySelectorAll(a.selectorText);
        } catch {}
        if (!(!i || !xo(i, l))) {
          var u = a.style.getPropertyValue("top"),
            f = a.style.getPropertyValue("bottom");
          if ((u && u !== "auto") || (f && f !== "auto")) return !0;
        }
      }
  }
  return !1;
};
kt.needsCentering = function (l) {
  var e = window.getComputedStyle(l);
  return e.position !== "absolute" || (l.style.top !== "auto" && l.style.top !== "") || (l.style.bottom !== "auto" && l.style.bottom !== "")
    ? !1
    : !kt.isInlinePositionSetByStylesheet(l);
};
kt.forceRegisterDialog = function (l) {
  if ((window.HTMLDialogElement || l.showModal, l.localName !== "dialog"))
    throw new Error("Failed to register dialog: The element is not a dialog.");
  new wo(l);
};
kt.registerDialog = function (l) {
  l.showModal || kt.forceRegisterDialog(l);
};
kt.DialogManager = function () {
  this.pendingDialogStack = [];
  var l = this.checkDOM_.bind(this);
  ((this.overlay = document.createElement("div")),
    (this.overlay.className = "_dialog_overlay"),
    this.overlay.addEventListener(
      "click",
      function (e) {
        ((this.forwardTab_ = void 0), e.stopPropagation(), l([]));
      }.bind(this),
    ),
    (this.handleKey_ = this.handleKey_.bind(this)),
    (this.handleFocus_ = this.handleFocus_.bind(this)),
    (this.zIndexLow_ = 1e5),
    (this.zIndexHigh_ = 1e5 + 150),
    (this.forwardTab_ = void 0),
    "MutationObserver" in window &&
      (this.mo_ = new MutationObserver(function (e) {
        var t = [];
        (e.forEach(function (s) {
          for (var n = 0, a; (a = s.removedNodes[n]); ++n) {
            if (a instanceof Element) a.localName === "dialog" && t.push(a);
            else continue;
            t = t.concat(a.querySelectorAll("dialog"));
          }
        }),
          t.length && l(t));
      })));
};
kt.DialogManager.prototype.blockDocument = function () {
  (document.documentElement.addEventListener("focus", this.handleFocus_, !0),
    document.addEventListener("keydown", this.handleKey_),
    this.mo_ && this.mo_.observe(document, { childList: !0, subtree: !0 }));
};
kt.DialogManager.prototype.unblockDocument = function () {
  (document.documentElement.removeEventListener("focus", this.handleFocus_, !0),
    document.removeEventListener("keydown", this.handleKey_),
    this.mo_ && this.mo_.disconnect());
};
kt.DialogManager.prototype.updateStacking = function () {
  for (var l = this.zIndexHigh_, e = 0, t; (t = this.pendingDialogStack[e]); ++e)
    (t.updateZIndex(--l, --l), e === 0 && (this.overlay.style.zIndex = --l));
  var s = this.pendingDialogStack[0];
  if (s) {
    var n = s.dialog.parentNode || document.body;
    n.appendChild(this.overlay);
  } else this.overlay.parentNode && this.overlay.parentNode.removeChild(this.overlay);
};
kt.DialogManager.prototype.containedByTopDialog_ = function (l) {
  for (; (l = Rn(l)); ) {
    for (var e = 0, t; (t = this.pendingDialogStack[e]); ++e) if (t.dialog === l) return e === 0;
    l = l.parentElement;
  }
  return !1;
};
kt.DialogManager.prototype.handleFocus_ = function (l) {
  var e = l.composedPath ? l.composedPath()[0] : l.target;
  if (
    !this.containedByTopDialog_(e) &&
    document.activeElement !== document.documentElement &&
    (l.preventDefault(), l.stopPropagation(), _o(e), this.forwardTab_ !== void 0)
  ) {
    var t = this.pendingDialogStack[0],
      s = t.dialog,
      n = s.compareDocumentPosition(e);
    return (
      n & Node.DOCUMENT_POSITION_PRECEDING &&
        (this.forwardTab_ ? t.focus_() : e !== document.documentElement && document.documentElement.focus()),
      !1
    );
  }
};
kt.DialogManager.prototype.handleKey_ = function (l) {
  if (((this.forwardTab_ = void 0), l.keyCode === 27)) {
    (l.preventDefault(), l.stopPropagation());
    var e = new An("cancel", { bubbles: !1, cancelable: !0 }),
      t = this.pendingDialogStack[0];
    t && vo(t.dialog, e) && t.dialog.close();
  } else l.keyCode === 9 && (this.forwardTab_ = !l.shiftKey);
};
kt.DialogManager.prototype.checkDOM_ = function (l) {
  var e = this.pendingDialogStack.slice();
  e.forEach(function (t) {
    l.indexOf(t.dialog) !== -1 ? t.downgradeModal() : t.maybeHideModal();
  });
};
kt.DialogManager.prototype.pushDialog = function (l) {
  var e = (this.zIndexHigh_ - this.zIndexLow_) / 2 - 1;
  return this.pendingDialogStack.length >= e
    ? !1
    : (this.pendingDialogStack.unshift(l) === 1 && this.blockDocument(), this.updateStacking(), !0);
};
kt.DialogManager.prototype.removeDialog = function (l) {
  var e = this.pendingDialogStack.indexOf(l);
  e !== -1 && (this.pendingDialogStack.splice(e, 1), this.pendingDialogStack.length === 0 && this.unblockDocument(), this.updateStacking());
};
kt.dm = new kt.DialogManager();
kt.formSubmitter = null;
kt.imagemapUseValue = null;
if (window.HTMLDialogElement === void 0) {
  var Ri = document.createElement("form");
  if ((Ri.setAttribute("method", "dialog"), Ri.method !== "dialog")) {
    var wn = Object.getOwnPropertyDescriptor(HTMLFormElement.prototype, "method");
    if (wn) {
      var Zo = wn.get;
      wn.get = function () {
        return _i(this) ? "dialog" : Zo.call(this);
      };
      var $o = wn.set;
      ((wn.set = function (l) {
        return typeof l == "string" && l.toLowerCase() === "dialog" ? this.setAttribute("method", l) : $o.call(this, l);
      }),
        Object.defineProperty(HTMLFormElement.prototype, "method", wn));
    }
  }
  (document.addEventListener(
    "click",
    function (l) {
      if (((kt.formSubmitter = null), (kt.imagemapUseValue = null), !l.defaultPrevented)) {
        var e = l.target;
        if ("composedPath" in l) {
          var t = l.composedPath();
          e = t.shift() || e;
        }
        if (!(!e || !_i(e.form))) {
          var s = e.type === "submit" && ["button", "input"].indexOf(e.localName) > -1;
          if (!s) {
            if (!(e.localName === "input" && e.type === "image")) return;
            kt.imagemapUseValue = l.offsetX + "," + l.offsetY;
          }
          var n = Rn(e);
          !n || (kt.formSubmitter = e);
        }
      }
    },
    !1,
  ),
    document.addEventListener("submit", function (l) {
      var e = l.target,
        t = Rn(e);
      if (!t) {
        var s = qo(l),
          n = (s && s.getAttribute("formmethod")) || e.getAttribute("method");
        n === "dialog" && l.preventDefault();
      }
    }));
  var ea = HTMLFormElement.prototype.submit,
    ta = function () {
      if (!_i(this)) return ea.call(this);
      var l = Rn(this);
      l && l.close();
    };
  HTMLFormElement.prototype.submit = ta;
}
function Ii(l) {
  let e;
  return {
    c() {
      e = C(l[1]);
    },
    m(t, s) {
      g(t, e, s);
    },
    p(t, s) {
      s & 2 && Z(e, t[1]);
    },
    d(t) {
      t && m(e);
    },
  };
}
function Di(l) {
  let e, t, s;
  return {
    c() {
      ((e = c("button")),
        (e.textContent = "\u2716"),
        r(e, "type", "button"),
        r(e, "class", "close-btn svelte-vhyrmr"),
        r(e, "aria-label", "Close"));
    },
    m(n, a) {
      (g(n, e, a), t || ((s = ce(e, "click", l[6])), (t = !0)));
    },
    p: qe,
    d(n) {
      (n && m(e), (t = !1), s());
    },
  };
}
function la(l) {
  let e,
    t,
    s,
    n,
    a,
    i,
    u,
    f,
    h,
    d,
    p,
    v,
    _ = l[1] && Ii(l),
    b = l[0] && Di(l);
  const w = l[12].default,
    y = So(w, l, l[11], null);
  return {
    c() {
      ((e = c("dialog")),
        (t = c("header")),
        _ && _.c(),
        (s = S()),
        (n = c("div")),
        (a = C(l[2])),
        (i = S()),
        b && b.c(),
        (u = S()),
        (f = c("div")),
        y && y.c(),
        r(n, "class", "title svelte-vhyrmr"),
        r(t, "class", "svelte-vhyrmr"),
        r(f, "class", "dialog-body svelte-vhyrmr"),
        r(e, "class", (h = Vs(l[4]) + " svelte-vhyrmr")));
    },
    m(A, q) {
      (g(A, e, q),
        o(e, t),
        _ && _.m(t, null),
        o(t, s),
        o(t, n),
        o(n, a),
        o(t, i),
        b && b.m(t, null),
        o(e, u),
        o(e, f),
        y && y.m(f, null),
        l[13](e),
        (d = !0),
        p || ((v = [ce(e, "close", l[5]), ce(e, "cancel", l[7])]), (p = !0)));
    },
    p(A, [q]) {
      (A[1] ? (_ ? _.p(A, q) : ((_ = Ii(A)), _.c(), _.m(t, s))) : _ && (_.d(1), (_ = null)),
        (!d || q & 4) && Z(a, A[2]),
        A[0] ? (b ? b.p(A, q) : ((b = Di(A)), b.c(), b.m(t, null))) : b && (b.d(1), (b = null)),
        y && y.p && (!d || q & 2048) && Mo(y, w, A, A[11], d ? ko(w, A[11], q, null) : Eo(A[11]), null),
        (!d || (q & 16 && h !== (h = Vs(A[4]) + " svelte-vhyrmr"))) && r(e, "class", h));
    },
    i(A) {
      d || (ve(y, A), (d = !0));
    },
    o(A) {
      (be(y, A), (d = !1));
    },
    d(A) {
      (A && m(e), _ && _.d(), b && b.d(), y && y.d(A), l[13](null), (p = !1), Zt(v));
    },
  };
}
function sa(l, e, t) {
  let s,
    { $$slots: n = {}, $$scope: a } = e,
    { canClose: i = !0 } = e,
    { className: u = "" } = e,
    { dialog: f = void 0 } = e,
    { icon: h = void 0 } = e,
    { title: d } = e,
    { data: p = {} } = e,
    v;
  const _ = $r();
  Zr(() => {
    (kt.registerDialog(v), t(8, (f = v)));
  });
  function b() {
    _("close");
  }
  function w() {
    v.close();
  }
  function y(q) {
    i || q.preventDefault();
  }
  function A(q) {
    dt[q ? "unshift" : "push"](() => {
      ((v = q), t(3, v));
    });
  }
  return (
    (l.$$set = (q) => {
      ("canClose" in q && t(0, (i = q.canClose)),
        "className" in q && t(9, (u = q.className)),
        "dialog" in q && t(8, (f = q.dialog)),
        "icon" in q && t(1, (h = q.icon)),
        "title" in q && t(2, (d = q.title)),
        "data" in q && t(10, (p = q.data)),
        "$$scope" in q && t(11, (a = q.$$scope)));
    }),
    (l.$$.update = () => {
      l.$$.dirty & 512 && t(4, (s = "dialog" + (u ? " " + u : "")));
    }),
    [i, h, d, v, s, b, w, y, f, u, p, a, n, A]
  );
}
class Pl extends Dl {
  constructor(e) {
    (super(), Il(this, e, sa, la, Ll, { canClose: 0, className: 9, dialog: 8, icon: 1, title: 2, data: 10 }));
  }
}
const na = "/assets/helpicon.6e85585d.svg";
function ia(l) {
  let e, t, s, n, a, i, u;
  return {
    c() {
      ((e = c("button")),
        (t = c("img")),
        mn(t.src, (s = na)) || r(t, "src", s),
        r(t, "alt", ""),
        r(t, "height", "20"),
        r(t, "class", "svelte-rl37t8"),
        r(e, "type", "button"),
        r(e, "class", "helpbutton svelte-rl37t8"),
        r(e, "aria-label", (n = "Help: " + l[0])),
        r(e, "title", (a = "Help: " + l[0])),
        ti(e, "spaced", l[1]));
    },
    m(f, h) {
      (g(f, e, h), o(e, t), i || ((u = ce(e, "click", l[2])), (i = !0)));
    },
    p(f, [h]) {
      (h & 1 && n !== (n = "Help: " + f[0]) && r(e, "aria-label", n),
        h & 1 && a !== (a = "Help: " + f[0]) && r(e, "title", a),
        h & 2 && ti(e, "spaced", f[1]));
    },
    i: qe,
    o: qe,
    d(f) {
      (f && m(e), (i = !1), u());
    },
  };
}
function ra(l, e, t) {
  let { topic: s = "" } = e,
    { spaced: n = !0 } = e;
  function a(i) {
    qi.call(this, l, i);
  }
  return (
    (l.$$set = (i) => {
      ("topic" in i && t(0, (s = i.topic)), "spaced" in i && t(1, (n = i.spaced)));
    }),
    [s, n, a]
  );
}
class Rl extends Dl {
  constructor(e) {
    (super(), Il(this, e, ra, ia, Ll, { topic: 0, spaced: 1 }));
  }
}
function oa(l) {
  let e, t, s, n, a, i, u, f, h, d, p, v, _, b, w;
  return {
    c() {
      ((e = C("Select partial if only a portion of the complete circular sequence is entered.")),
        (t = c("br")),
        (s = S()),
        (n = c("br")),
        (a = C(`
  		Selecting the "Partial" checkbox tells NEBaseChanger that it should not assume `)),
        (i = c("br")),
        (u = C(`
			a complete circular sequence, allowing the entry of a much shorter sequence.  `)),
        (f = c("br")),
        (h = C(`
			However, partial sequences will need to include sufficient sequence flanking the mutation site, `)),
        (d = c("br")),
        (p = C(`
			currently set at a minimum of 35 nt 5\u2032 and 25 nt 3\u2032. Mutations will be prohibited `)),
        (v = c("br")),
        (_ = C(`
			from being too close to the ends of the provided partial sequence fragment `)),
        (b = c("br")),
        (w = C(`
			and cannot span the ends of the fragment.`)),
        r(t, "class", "svelte-134jeqo"),
        r(n, "class", "svelte-134jeqo"),
        r(i, "class", "svelte-134jeqo"),
        r(f, "class", "svelte-134jeqo"),
        r(d, "class", "svelte-134jeqo"),
        r(v, "class", "svelte-134jeqo"),
        r(b, "class", "svelte-134jeqo"));
    },
    m(y, A) {
      (g(y, e, A),
        g(y, t, A),
        g(y, s, A),
        g(y, n, A),
        g(y, a, A),
        g(y, i, A),
        g(y, u, A),
        g(y, f, A),
        g(y, h, A),
        g(y, d, A),
        g(y, p, A),
        g(y, v, A),
        g(y, _, A),
        g(y, b, A),
        g(y, w, A));
    },
    p: qe,
    d(y) {
      (y && m(e),
        y && m(t),
        y && m(s),
        y && m(n),
        y && m(a),
        y && m(i),
        y && m(u),
        y && m(f),
        y && m(h),
        y && m(d),
        y && m(p),
        y && m(v),
        y && m(_),
        y && m(b),
        y && m(w));
    },
  };
}
function Oi(l) {
  let e, t;
  return {
    c() {
      ((e = c("div")), (t = C(l[13])), r(e, "class", "warn svelte-134jeqo"), r(e, "id", "vecloaderror"));
    },
    m(s, n) {
      (g(s, e, n), o(e, t));
    },
    p(s, n) {
      n[0] & 8192 && Z(t, s[13]);
    },
    d(s) {
      s && m(e);
    },
  };
}
function ji(l) {
  let e;
  return {
    c() {
      ((e = c("span")),
        (e.textContent = "Invalid sequence - NON-CANONICAL BASES - please re-enter."),
        r(e, "class", "warn svelte-134jeqo"));
    },
    m(t, s) {
      g(t, e, s);
    },
    d(t) {
      t && m(e);
    },
  };
}
function Bi(l) {
  let e, t, s, n;
  return {
    c() {
      ((e = c("div")),
        (t = c("div")),
        (s = c("span")),
        (n = C(l[10])),
        r(s, "class", "notice svelte-134jeqo"),
        r(t, "class", "col svelte-134jeqo"),
        r(e, "class", "row svelte-134jeqo"));
    },
    m(a, i) {
      (g(a, e, i), o(e, t), o(t, s), o(s, n));
    },
    p(a, i) {
      i[0] & 1024 && Z(n, a[10]);
    },
    d(a) {
      a && m(e);
    },
  };
}
function aa(l) {
  let e,
    t,
    s,
    n,
    a,
    i,
    u,
    f,
    h,
    d,
    p,
    v,
    _,
    b,
    w,
    y,
    A,
    q,
    T,
    N,
    M,
    k,
    E,
    P,
    G,
    O,
    K,
    D,
    L,
    R,
    Q,
    ae,
    W,
    x,
    H,
    U,
    le,
    J,
    ie,
    z,
    te,
    pe,
    Y,
    ue,
    he,
    de,
    _e,
    ne,
    $,
    fe,
    we,
    j,
    F,
    ee,
    V,
    me,
    _t,
    Je,
    bt,
    ut,
    it,
    Be,
    We,
    Ie,
    Ce,
    De,
    at,
    Ve,
    $t,
    rt,
    Xt,
    Xe,
    Oe,
    He,
    Ke,
    ke,
    Ue,
    nl,
    At,
    Gt,
    pt,
    Ee,
    Qe,
    ht,
    tt,
    pl,
    Kl,
    Mt,
    hl,
    Et,
    bs,
    kl,
    qs,
    Bt,
    ws,
    Vt,
    Ft,
    Yl,
    Ol,
    Wl,
    ml,
    jl,
    Gl,
    Nt,
    zl,
    rl,
    rs,
    el,
    os,
    tl,
    qt,
    gl,
    Fl,
    Xl,
    Ze,
    vl,
    Ql,
    xl,
    lt,
    st,
    Ml,
    Kt,
    ys,
    Jl,
    as,
    Pt,
    Ht,
    Zl,
    cs,
    Qt,
    As,
    _l,
    Rt,
    Lt,
    Bl,
    bl,
    Ut,
    il,
    ll,
    ql,
    Hl,
    Ul,
    us,
    It,
    Ks,
    zs,
    ol,
    Gs,
    xt,
    tn,
    Fs,
    wl,
    ln,
    $l,
    fs,
    Rs,
    es,
    Cs,
    ds,
    yl,
    un,
    Vl,
    Ys,
    ts,
    Ts,
    Is,
    ps,
    Al,
    Xs,
    sl,
    al,
    Ss,
    Cl,
    ks,
    Ds,
    hs,
    Qs,
    ms = l[0].seq.length + "",
    xs,
    Ms,
    sn,
    Os,
    js,
    Yt,
    cl,
    ls,
    nn,
    gs,
    rn,
    Bs,
    Hs,
    Js,
    fn;
  function Zs(Ae) {
    l[20](Ae);
  }
  let $s = { title: "Partial Sequence Help ", $$slots: { default: [oa] }, $$scope: { ctx: l } };
  (l[12] !== void 0 && ($s.dialog = l[12]), (e = new Pl({ props: $s })), dt.push(() => zt(e, "dialog", Zs)));
  let ul = l[13] && Oi(l);
  ((ts = new Rl({ props: { topic: "Partial Sequence", spaced: !1 } })), ts.$on("click", l[28]));
  let Tl = l[9] && ji(),
    El = l[10] && Bi(l);
  return {
    c() {
      (Re(e.$$.fragment),
        (s = S()),
        (n = c("div")),
        (a = c("div")),
        (i = c("form")),
        (u = c("input")),
        (f = C(`
	    	 or paste/type sequence below.`)),
        (h = S()),
        (d = c("div")),
        (p = c("select")),
        (v = c("option")),
        (v.textContent = "Sample Vectors"),
        (_ = c("option")),
        (_.textContent = "lambda_gt11"),
        (b = c("option")),
        (b.textContent = "LITMUS_U"),
        (w = c("option")),
        (w.textContent = "LITMUS28"),
        (y = c("option")),
        (y.textContent = "LITMUS28i"),
        (A = c("option")),
        (A.textContent = "LITMUS29"),
        (q = c("option")),
        (q.textContent = "LITMUS38"),
        (T = c("option")),
        (T.textContent = "LITMUS38i"),
        (N = c("option")),
        (N.textContent = "LITMUS39"),
        (M = c("option")),
        (M.textContent = "M13KE"),
        (k = c("option")),
        (k.textContent = "M13KO7"),
        (E = c("option")),
        (E.textContent = "M13mp18"),
        (P = c("option")),
        (P.textContent = "pACP-GPI"),
        (G = c("option")),
        (G.textContent = "pACPm"),
        (O = c("option")),
        (O.textContent = "pACYC177"),
        (K = c("option")),
        (K.textContent = "pACYC184"),
        (D = c("option")),
        (D.textContent = "pAd2-AvrII"),
        (L = c("option")),
        (L.textContent = "pAd2-BsaBI"),
        (R = c("option")),
        (R.textContent = "pASI3"),
        (Q = c("option")),
        (Q.textContent = "pBAD24"),
        (ae = c("option")),
        (ae.textContent = "pBeloBAC11"),
        (W = c("option")),
        (W.textContent = "pBR322"),
        (x = c("option")),
        (x.textContent = "pCLIP-H2B"),
        (H = c("option")),
        (H.textContent = "pCLIP-NK1R"),
        (U = c("option")),
        (U.textContent = "pCLIPm"),
        (le = c("option")),
        (le.textContent = "pCMV-GLuc"),
        (J = c("option")),
        (J.textContent = "pCYB1"),
        (ie = c("option")),
        (ie.textContent = "pCYB2"),
        (z = c("option")),
        (z.textContent = "pCYB3"),
        (te = c("option")),
        (te.textContent = "pCYB4"),
        (pe = c("option")),
        (pe.textContent = "pET11c"),
        (Y = c("option")),
        (Y.textContent = "pET-21(+)"),
        (ue = c("option")),
        (ue.textContent = "pET21a(+)"),
        (he = c("option")),
        (he.textContent = "pFOS1"),
        (de = c("option")),
        (de.textContent = "pGLO"),
        (_e = c("option")),
        (_e.textContent = "pGLuc-Basic"),
        (ne = c("option")),
        (ne.textContent = "pGLuc-Mini-TK"),
        ($ = c("option")),
        ($.textContent = "pGPS1.1"),
        (fe = c("option")),
        (fe.textContent = "pGPS2.1"),
        (we = c("option")),
        (we.textContent = "pGPS3"),
        (j = c("option")),
        (j.textContent = "pGPS4"),
        (F = c("option")),
        (F.textContent = "pGPS5"),
        (ee = c("option")),
        (ee.textContent = "pGT-N28"),
        (V = c("option")),
        (V.textContent = "pGT-N29"),
        (me = c("option")),
        (me.textContent = "pGT-N38"),
        (_t = c("option")),
        (_t.textContent = "pGT-N39"),
        (Je = c("option")),
        (Je.textContent = "PhiX174"),
        (bt = c("option")),
        (bt.textContent = "pKLAC-malE"),
        (ut = c("option")),
        (ut.textContent = "FJ010198"),
        (it = c("option")),
        (it.textContent = "pKLAC1"),
        (Be = c("option")),
        (Be.textContent = "EU196354"),
        (We = c("option")),
        (We.textContent = "pKYB1"),
        (Ie = c("option")),
        (Ie.textContent = "pMAL-c"),
        (Ce = c("option")),
        (Ce.textContent = "pMAL-c2"),
        (De = c("option")),
        (De.textContent = "pMAL-c2E"),
        (at = c("option")),
        (at.textContent = "pMAL-c2G"),
        (Ve = c("option")),
        (Ve.textContent = "pMAL-c2X"),
        ($t = c("option")),
        ($t.textContent = "pMAL-c4E"),
        (rt = c("option")),
        (rt.textContent = "pMAL-c4G"),
        (Xt = c("option")),
        (Xt.textContent = "pMAL-c4X"),
        (Xe = c("option")),
        (Xe.textContent = "pMAL-c5E"),
        (Oe = c("option")),
        (Oe.textContent = "pMAL-c5G"),
        (He = c("option")),
        (He.textContent = "pMAL-c5X"),
        (Ke = c("option")),
        (Ke.textContent = "pMAL-cRI"),
        (ke = c("option")),
        (ke.textContent = "pMAL-p"),
        (Ue = c("option")),
        (Ue.textContent = "pMAL-p2"),
        (nl = c("option")),
        (nl.textContent = "pMAL-p2E"),
        (At = c("option")),
        (At.textContent = "pMAL-p2G"),
        (Gt = c("option")),
        (Gt.textContent = "pMAL-p2X"),
        (pt = c("option")),
        (pt.textContent = "pMAL-p4E"),
        (Ee = c("option")),
        (Ee.textContent = "pMAL-p4G"),
        (Qe = c("option")),
        (Qe.textContent = "pMAL-p4X"),
        (ht = c("option")),
        (ht.textContent = "pMAL-p5E"),
        (tt = c("option")),
        (tt.textContent = "pMAL-p5G"),
        (pl = c("option")),
        (pl.textContent = "pMAL-p5X"),
        (Kl = c("option")),
        (Kl.textContent = "pMAL-pIII"),
        (Mt = c("option")),
        (Mt.textContent = "pMCP-GPI"),
        (hl = c("option")),
        (hl.textContent = "pMCPm"),
        (Et = c("option")),
        (Et.textContent = "pMXB10"),
        (bs = c("option")),
        (bs.textContent = "pMYB5"),
        (kl = c("option")),
        (kl.textContent = "pNEB193"),
        (qs = c("option")),
        (qs.textContent = "pNEB205a"),
        (Bt = c("option")),
        (Bt.textContent = "pNEB206a"),
        (ws = c("option")),
        (ws.textContent = "pNEBR-R1"),
        (Vt = c("option")),
        (Vt.textContent = "pNEBR-R1Puro"),
        (Ft = c("option")),
        (Ft.textContent = "pNEBR-X1"),
        (Yl = c("option")),
        (Yl.textContent = "pNEBR-X1GLuc"),
        (Ol = c("option")),
        (Ol.textContent = "pNEBR-X1Hygro"),
        (Wl = c("option")),
        (Wl.textContent = "PRS303"),
        (ml = c("option")),
        (ml.textContent = "PRS304"),
        (jl = c("option")),
        (jl.textContent = "PRS305"),
        (Gl = c("option")),
        (Gl.textContent = "PRS306"),
        (Nt = c("option")),
        (Nt.textContent = "PRS313"),
        (zl = c("option")),
        (zl.textContent = "PRS314"),
        (rl = c("option")),
        (rl.textContent = "PRS315"),
        (rs = c("option")),
        (rs.textContent = "PRS316"),
        (el = c("option")),
        (el.textContent = "pRS413"),
        (os = c("option")),
        (os.textContent = "pRS414"),
        (tl = c("option")),
        (tl.textContent = "pRS415"),
        (qt = c("option")),
        (qt.textContent = "pRS416"),
        (gl = c("option")),
        (gl.textContent = "pSNAP-ADRB2"),
        (Fl = c("option")),
        (Fl.textContent = "pSNAP-H2B"),
        (Xl = c("option")),
        (Xl.textContent = "pSNAPm"),
        (Ze = c("option")),
        (Ze.textContent = "pT7-SNAP"),
        (vl = c("option")),
        (vl.textContent = "pTK-GLuc"),
        (Ql = c("option")),
        (Ql.textContent = "pTWIN-MBP"),
        (xl = c("option")),
        (xl.textContent = "pTWIN1"),
        (lt = c("option")),
        (lt.textContent = "pTWIN2"),
        (st = c("option")),
        (st.textContent = "pTXB1"),
        (Ml = c("option")),
        (Ml.textContent = "pTXB3"),
        (Kt = c("option")),
        (Kt.textContent = "pTYB1"),
        (ys = c("option")),
        (ys.textContent = "pTYB11"),
        (Jl = c("option")),
        (Jl.textContent = "pTYB12"),
        (as = c("option")),
        (as.textContent = "pTYB2"),
        (Pt = c("option")),
        (Pt.textContent = "pTYB21"),
        (Ht = c("option")),
        (Ht.textContent = "pTYB22"),
        (Zl = c("option")),
        (Zl.textContent = "pTYB3"),
        (cs = c("option")),
        (cs.textContent = "pTYB4"),
        (Qt = c("option")),
        (Qt.textContent = "SYNPUC18CV"),
        (As = c("option")),
        (As.textContent = "pUC19"),
        (_l = c("option")),
        (_l.textContent = "pUPS"),
        (Rt = c("option")),
        (Rt.textContent = "PureExpress DHFR Control Template"),
        (Lt = c("option")),
        (Lt.textContent = "SV40"),
        (Bl = c("option")),
        (Bl.textContent = "X75457"),
        (bl = c("option")),
        (bl.textContent = "X75455"),
        (Ut = c("option")),
        (Ut.textContent = "X75456"),
        (il = c("option")),
        (il.textContent = "YEp24"),
        (ll = c("option")),
        (ll.textContent = "X75458"),
        (ql = c("option")),
        (ql.textContent = "X75460"),
        (Hl = c("option")),
        (Hl.textContent = "X75459"),
        (Ul = c("option")),
        (Ul.textContent = "X75463"),
        (us = c("option")),
        (us.textContent = "X75461"),
        (It = c("option")),
        (It.textContent = "X75462"),
        (Ks = S()),
        ul && ul.c(),
        (zs = S()),
        (ol = c("div")),
        (Gs = S()),
        (xt = c("br")),
        (tn = S()),
        (Fs = c("div")),
        (wl = c("div")),
        (ln = C(`\xA0\xA0\xA0
	    	`)),
        ($l = c("span")),
        (fs = c("input")),
        (Rs = S()),
        (es = c("label")),
        (es.textContent = "Use Reverse Complement"),
        (Cs = C(`
        \xA0\xA0\xA0
        `)),
        (ds = c("span")),
        (yl = c("input")),
        (un = S()),
        (Vl = c("label")),
        (Vl.textContent = "Partial sequence"),
        (Ys = S()),
        Re(ts.$$.fragment),
        (Ts = C(`
        \xA0\xA0\xA0
	    	`)),
        (Is = c("span")),
        (ps = c("button")),
        (ps.innerHTML = `<span class="glyphicon glyphicon-refresh svelte-134jeqo">x</span>
          Clear`),
        (Al = S()),
        Tl && Tl.c(),
        (Xs = S()),
        El && El.c(),
        (sl = S()),
        (al = c("div")),
        (Ss = c("div")),
        (Cl = c("textarea")),
        (ks = S()),
        (Ds = c("div")),
        (hs = c("div")),
        (Qs = C("Length: ")),
        (xs = C(ms)),
        (Ms = C(" bp \xA0\xA0")),
        (sn = S()),
        (Os = c("div")),
        (js = c("div")),
        (Yt = c("label")),
        (Yt.textContent = "Sequence Name/ID:"),
        (cl = S()),
        (ls = c("input")),
        (nn = S()),
        (gs = c("div")),
        (gs.innerHTML =
          '<div id="design" class="svelte-134jeqo"><div id="inputsequence" unselectable="on" class="svelte-134jeqo"></div></div>'),
        (rn = S()),
        (Bs = c("div")),
        (Bs.innerHTML = '<div class="col svelte-134jeqo"></div>'),
        r(u, "id", "fileinput"),
        r(u, "type", "file"),
        r(u, "aria-label", "Load a sequence file"),
        r(u, "class", "svelte-134jeqo"),
        r(i, "id", "fileform"),
        r(i, "name", "fileform"),
        r(i, "class", "svelte-134jeqo"),
        r(a, "class", "col svelte-134jeqo"),
        (v.__value = "unselected"),
        (v.value = v.__value),
        r(v, "class", "svelte-134jeqo"),
        (_.__value = "Lambda_gt11.fasta.txt"),
        (_.value = _.__value),
        r(_, "class", "svelte-134jeqo"),
        (b.__value = "LITMUS-U.fasta.txt"),
        (b.value = b.__value),
        r(b, "class", "svelte-134jeqo"),
        (w.__value = "LITMUS28.fasta.txt"),
        (w.value = w.__value),
        r(w, "class", "svelte-134jeqo"),
        (y.__value = "LITMUS28i.fasta.txt"),
        (y.value = y.__value),
        r(y, "class", "svelte-134jeqo"),
        (A.__value = "LITMUS29.fasta.txt"),
        (A.value = A.__value),
        r(A, "class", "svelte-134jeqo"),
        (q.__value = "LITMUS38.fasta.txt"),
        (q.value = q.__value),
        r(q, "class", "svelte-134jeqo"),
        (T.__value = "LITMUS38i.fasta.txt"),
        (T.value = T.__value),
        r(T, "class", "svelte-134jeqo"),
        (N.__value = "LITMUS39.fasta.txt"),
        (N.value = N.__value),
        r(N, "class", "svelte-134jeqo"),
        (M.__value = "M13KE.fasta.txt"),
        (M.value = M.__value),
        r(M, "class", "svelte-134jeqo"),
        (k.__value = "M13KO7.fasta.txt"),
        (k.value = k.__value),
        r(k, "class", "svelte-134jeqo"),
        (E.__value = "M13mp18.fasta.txt"),
        (E.value = E.__value),
        r(E, "class", "svelte-134jeqo"),
        (P.__value = "pACP-GPI.fasta.txt"),
        (P.value = P.__value),
        r(P, "class", "svelte-134jeqo"),
        (G.__value = "pACPm.fasta.txt"),
        (G.value = G.__value),
        r(G, "class", "svelte-134jeqo"),
        (O.__value = "pACYC177.fasta.txt"),
        (O.value = O.__value),
        r(O, "class", "svelte-134jeqo"),
        (K.__value = "pACYC184.fasta.txt"),
        (K.value = K.__value),
        r(K, "class", "svelte-134jeqo"),
        (D.__value = "pAd2-AvrII.fasta.txt"),
        (D.value = D.__value),
        r(D, "class", "svelte-134jeqo"),
        (L.__value = "pAd2-BsaBI.fasta.txt"),
        (L.value = L.__value),
        r(L, "class", "svelte-134jeqo"),
        (R.__value = "pASI3.fasta.txt"),
        (R.value = R.__value),
        r(R, "class", "svelte-134jeqo"),
        (Q.__value = "pBAD24.fasta.txt"),
        (Q.value = Q.__value),
        r(Q, "class", "svelte-134jeqo"),
        (ae.__value = "pBeloBAC11.fasta.txt"),
        (ae.value = ae.__value),
        r(ae, "class", "svelte-134jeqo"),
        (W.__value = "pBR322.fasta.txt"),
        (W.value = W.__value),
        r(W, "class", "svelte-134jeqo"),
        (x.__value = "pCLIP-H2B.fasta.txt"),
        (x.value = x.__value),
        r(x, "class", "svelte-134jeqo"),
        (H.__value = "pCLIP-NK1R.fasta.txt"),
        (H.value = H.__value),
        r(H, "class", "svelte-134jeqo"),
        (U.__value = "pCLIPm.fasta.txt"),
        (U.value = U.__value),
        r(U, "class", "svelte-134jeqo"),
        (le.__value = "pCMV-GLuc.fasta.txt"),
        (le.value = le.__value),
        r(le, "class", "svelte-134jeqo"),
        (J.__value = "pCYB1.fasta.txt"),
        (J.value = J.__value),
        r(J, "class", "svelte-134jeqo"),
        (ie.__value = "pCYB2.fasta.txt"),
        (ie.value = ie.__value),
        r(ie, "class", "svelte-134jeqo"),
        (z.__value = "pCYB3.fasta.txt"),
        (z.value = z.__value),
        r(z, "class", "svelte-134jeqo"),
        (te.__value = "pCYB4.fasta.txt"),
        (te.value = te.__value),
        r(te, "class", "svelte-134jeqo"),
        (pe.__value = "pET11c.fasta.txt"),
        (pe.value = pe.__value),
        r(pe, "class", "svelte-134jeqo"),
        (Y.__value = "pet21.fasta.txt"),
        (Y.value = Y.__value),
        r(Y, "class", "svelte-134jeqo"),
        (ue.__value = "pet21a__.fasta.txt"),
        (ue.value = ue.__value),
        r(ue, "class", "svelte-134jeqo"),
        (he.__value = "pFOS1.fasta.txt"),
        (he.value = he.__value),
        r(he, "class", "svelte-134jeqo"),
        (de.__value = "pGLO.fasta.txt"),
        (de.value = de.__value),
        r(de, "class", "svelte-134jeqo"),
        (_e.__value = "pGLuc-Basic.fasta.txt"),
        (_e.value = _e.__value),
        r(_e, "class", "svelte-134jeqo"),
        (ne.__value = "pGLuc_Mini-TK.fasta.txt"),
        (ne.value = ne.__value),
        r(ne, "class", "svelte-134jeqo"),
        ($.__value = "pGPS1.1.fasta.txt"),
        ($.value = $.__value),
        r($, "class", "svelte-134jeqo"),
        (fe.__value = "pGPS2.1.fasta.txt"),
        (fe.value = fe.__value),
        r(fe, "class", "svelte-134jeqo"),
        (we.__value = "pGPS3.fasta.txt"),
        (we.value = we.__value),
        r(we, "class", "svelte-134jeqo"),
        (j.__value = "pGPS4.fasta.txt"),
        (j.value = j.__value),
        r(j, "class", "svelte-134jeqo"),
        (F.__value = "pGPS5.fasta.txt"),
        (F.value = F.__value),
        r(F, "class", "svelte-134jeqo"),
        (ee.__value = "pGT-N28.fasta.txt"),
        (ee.value = ee.__value),
        r(ee, "class", "svelte-134jeqo"),
        (V.__value = "pGT-N29.fasta.txt"),
        (V.value = V.__value),
        r(V, "class", "svelte-134jeqo"),
        (me.__value = "pGT-N38.fasta.txt"),
        (me.value = me.__value),
        r(me, "class", "svelte-134jeqo"),
        (_t.__value = "pGT-N39.fasta.txt"),
        (_t.value = _t.__value),
        r(_t, "class", "svelte-134jeqo"),
        (Je.__value = "PhiX174.fasta.txt"),
        (Je.value = Je.__value),
        r(Je, "class", "svelte-134jeqo"),
        (bt.__value = "pKLAC-malE.fasta.txt"),
        (bt.value = bt.__value),
        r(bt, "class", "svelte-134jeqo"),
        (ut.__value = "pKLAC1-GLuc.fasta.txt"),
        (ut.value = ut.__value),
        r(ut, "class", "svelte-134jeqo"),
        (it.__value = "pKLAC1.fasta.txt"),
        (it.value = it.__value),
        r(it, "class", "svelte-134jeqo"),
        (Be.__value = "pKLAC2.fasta.txt"),
        (Be.value = Be.__value),
        r(Be, "class", "svelte-134jeqo"),
        (We.__value = "pKYB1.fasta.txt"),
        (We.value = We.__value),
        r(We, "class", "svelte-134jeqo"),
        (Ie.__value = "pMAL-c.fasta.txt"),
        (Ie.value = Ie.__value),
        r(Ie, "class", "svelte-134jeqo"),
        (Ce.__value = "pMAL-c2.fasta.txt"),
        (Ce.value = Ce.__value),
        r(Ce, "class", "svelte-134jeqo"),
        (De.__value = "pMAL-c2E.fasta.txt"),
        (De.value = De.__value),
        r(De, "class", "svelte-134jeqo"),
        (at.__value = "pMAL-c2G.fasta.txt"),
        (at.value = at.__value),
        r(at, "class", "svelte-134jeqo"),
        (Ve.__value = "pMAL-c2X.fasta.txt"),
        (Ve.value = Ve.__value),
        r(Ve, "class", "svelte-134jeqo"),
        ($t.__value = "pMAL-c4E.fasta.txt"),
        ($t.value = $t.__value),
        r($t, "class", "svelte-134jeqo"),
        (rt.__value = "pMAL-c4G.fasta.txt"),
        (rt.value = rt.__value),
        r(rt, "class", "svelte-134jeqo"),
        (Xt.__value = "pMAL-c4X.fasta.txt"),
        (Xt.value = Xt.__value),
        r(Xt, "class", "svelte-134jeqo"),
        (Xe.__value = "pMAL-c5E.fasta.txt"),
        (Xe.value = Xe.__value),
        r(Xe, "class", "svelte-134jeqo"),
        (Oe.__value = "pMAL-c5G.fasta.txt"),
        (Oe.value = Oe.__value),
        r(Oe, "class", "svelte-134jeqo"),
        (He.__value = "pMAL-c5X.fasta.txt"),
        (He.value = He.__value),
        r(He, "class", "svelte-134jeqo"),
        (Ke.__value = "pMAL-cRI.fasta.txt"),
        (Ke.value = Ke.__value),
        r(Ke, "class", "svelte-134jeqo"),
        (ke.__value = "pMAL-p.fasta.txt"),
        (ke.value = ke.__value),
        r(ke, "class", "svelte-134jeqo"),
        (Ue.__value = "pMAL-p2.fasta.txt"),
        (Ue.value = Ue.__value),
        r(Ue, "class", "svelte-134jeqo"),
        (nl.__value = "pMAL-p2E.fasta.txt"),
        (nl.value = nl.__value),
        r(nl, "class", "svelte-134jeqo"),
        (At.__value = "pMAL-p2G.fasta.txt"),
        (At.value = At.__value),
        r(At, "class", "svelte-134jeqo"),
        (Gt.__value = "pMAL-p2X.fasta.txt"),
        (Gt.value = Gt.__value),
        r(Gt, "class", "svelte-134jeqo"),
        (pt.__value = "pMAL-p4E.fasta.txt"),
        (pt.value = pt.__value),
        r(pt, "class", "svelte-134jeqo"),
        (Ee.__value = "pMAL-p4G.fasta.txt"),
        (Ee.value = Ee.__value),
        r(Ee, "class", "svelte-134jeqo"),
        (Qe.__value = "pMAL-p4X.fasta.txt"),
        (Qe.value = Qe.__value),
        r(Qe, "class", "svelte-134jeqo"),
        (ht.__value = "pMAL-p5E.fasta.txt"),
        (ht.value = ht.__value),
        r(ht, "class", "svelte-134jeqo"),
        (tt.__value = "pMAL-p5G.fasta.txt"),
        (tt.value = tt.__value),
        r(tt, "class", "svelte-134jeqo"),
        (pl.__value = "pMAL-p5X.fasta.txt"),
        (pl.value = pl.__value),
        r(pl, "class", "svelte-134jeqo"),
        (Kl.__value = "pMAL-pIII.fasta.txt"),
        (Kl.value = Kl.__value),
        r(Kl, "class", "svelte-134jeqo"),
        (Mt.__value = "pMCP-GPI.fasta.txt"),
        (Mt.value = Mt.__value),
        r(Mt, "class", "svelte-134jeqo"),
        (hl.__value = "pMCPm.fasta.txt"),
        (hl.value = hl.__value),
        r(hl, "class", "svelte-134jeqo"),
        (Et.__value = "pMXB10.fasta.txt"),
        (Et.value = Et.__value),
        r(Et, "class", "svelte-134jeqo"),
        (bs.__value = "pMYB5.fasta.txt"),
        (bs.value = bs.__value),
        r(bs, "class", "svelte-134jeqo"),
        (kl.__value = "pNEB193.fasta.txt"),
        (kl.value = kl.__value),
        r(kl, "class", "svelte-134jeqo"),
        (qs.__value = "pNEB205a.fasta.txt"),
        (qs.value = qs.__value),
        r(qs, "class", "svelte-134jeqo"),
        (Bt.__value = "pNEB206a.fasta.txt"),
        (Bt.value = Bt.__value),
        r(Bt, "class", "svelte-134jeqo"),
        (ws.__value = "pNEBR-R1.fasta.txt"),
        (ws.value = ws.__value),
        r(ws, "class", "svelte-134jeqo"),
        (Vt.__value = "pNEBR-R1Puro.fasta.txt"),
        (Vt.value = Vt.__value),
        r(Vt, "class", "svelte-134jeqo"),
        (Ft.__value = "pNEBR-X1.fasta.txt"),
        (Ft.value = Ft.__value),
        r(Ft, "class", "svelte-134jeqo"),
        (Yl.__value = "pNEBR-X1GLuc.fasta.txt"),
        (Yl.value = Yl.__value),
        r(Yl, "class", "svelte-134jeqo"),
        (Ol.__value = "pNEBR-X1Hygro.fasta.txt"),
        (Ol.value = Ol.__value),
        r(Ol, "class", "svelte-134jeqo"),
        (Wl.__value = "pRS303.fasta.txt"),
        (Wl.value = Wl.__value),
        r(Wl, "class", "svelte-134jeqo"),
        (ml.__value = "pRS304.fasta.txt"),
        (ml.value = ml.__value),
        r(ml, "class", "svelte-134jeqo"),
        (jl.__value = "pRS305.fasta.txt"),
        (jl.value = jl.__value),
        r(jl, "class", "svelte-134jeqo"),
        (Gl.__value = "pRS306.fasta.txt"),
        (Gl.value = Gl.__value),
        r(Gl, "class", "svelte-134jeqo"),
        (Nt.__value = "pRS313.fasta.txt"),
        (Nt.value = Nt.__value),
        r(Nt, "class", "svelte-134jeqo"),
        (zl.__value = "pRS314.fasta.txt"),
        (zl.value = zl.__value),
        r(zl, "class", "svelte-134jeqo"),
        (rl.__value = "pRS315.fasta.txt"),
        (rl.value = rl.__value),
        r(rl, "class", "svelte-134jeqo"),
        (rs.__value = "pRS316.fasta.txt"),
        (rs.value = rs.__value),
        r(rs, "class", "svelte-134jeqo"),
        (el.__value = "pRS413.fasta.txt"),
        (el.value = el.__value),
        r(el, "class", "svelte-134jeqo"),
        (os.__value = "pRS414.fasta.txt"),
        (os.value = os.__value),
        r(os, "class", "svelte-134jeqo"),
        (tl.__value = "pRS415.fasta.txt"),
        (tl.value = tl.__value),
        r(tl, "class", "svelte-134jeqo"),
        (qt.__value = "pRS416.fasta.txt"),
        (qt.value = qt.__value),
        r(qt, "class", "svelte-134jeqo"),
        (gl.__value = "pSNAP-ADRB2.fasta.txt"),
        (gl.value = gl.__value),
        r(gl, "class", "svelte-134jeqo"),
        (Fl.__value = "pSNAP-H2B.fasta.txt"),
        (Fl.value = Fl.__value),
        r(Fl, "class", "svelte-134jeqo"),
        (Xl.__value = "pSNAPm.fasta.txt"),
        (Xl.value = Xl.__value),
        r(Xl, "class", "svelte-134jeqo"),
        (Ze.__value = "pT7-SNAP.fasta.txt"),
        (Ze.value = Ze.__value),
        r(Ze, "class", "svelte-134jeqo"),
        (vl.__value = "pTK-GLuc.fasta.txt"),
        (vl.value = vl.__value),
        r(vl, "class", "svelte-134jeqo"),
        (Ql.__value = "pTWIN-MBP1.fasta.txt"),
        (Ql.value = Ql.__value),
        r(Ql, "class", "svelte-134jeqo"),
        (xl.__value = "pTWIN1.fasta.txt"),
        (xl.value = xl.__value),
        r(xl, "class", "svelte-134jeqo"),
        (lt.__value = "pTWIN2.fasta.txt"),
        (lt.value = lt.__value),
        r(lt, "class", "svelte-134jeqo"),
        (st.__value = "pTXB1.fasta.txt"),
        (st.value = st.__value),
        r(st, "class", "svelte-134jeqo"),
        (Ml.__value = "pTXB3.fasta.txt"),
        (Ml.value = Ml.__value),
        r(Ml, "class", "svelte-134jeqo"),
        (Kt.__value = "pTYB1.fasta.txt"),
        (Kt.value = Kt.__value),
        r(Kt, "class", "svelte-134jeqo"),
        (ys.__value = "pTYB11.fasta.txt"),
        (ys.value = ys.__value),
        r(ys, "class", "svelte-134jeqo"),
        (Jl.__value = "pTYB12.fasta.txt"),
        (Jl.value = Jl.__value),
        r(Jl, "class", "svelte-134jeqo"),
        (as.__value = "pTYB2.fasta.txt"),
        (as.value = as.__value),
        r(as, "class", "svelte-134jeqo"),
        (Pt.__value = "pTYB21.fasta.txt"),
        (Pt.value = Pt.__value),
        r(Pt, "class", "svelte-134jeqo"),
        (Ht.__value = "pTYB22.fasta.txt"),
        (Ht.value = Ht.__value),
        r(Ht, "class", "svelte-134jeqo"),
        (Zl.__value = "pTYB3.fasta.txt"),
        (Zl.value = Zl.__value),
        r(Zl, "class", "svelte-134jeqo"),
        (cs.__value = "pTYB4.fasta.txt"),
        (cs.value = cs.__value),
        r(cs, "class", "svelte-134jeqo"),
        (Qt.__value = "pUC18.fasta.txt"),
        (Qt.value = Qt.__value),
        r(Qt, "class", "svelte-134jeqo"),
        (As.__value = "pUC19.fasta.txt"),
        (As.value = As.__value),
        r(As, "class", "svelte-134jeqo"),
        (_l.__value = "pUPS.fasta.txt"),
        (_l.value = _l.__value),
        r(_l, "class", "svelte-134jeqo"),
        (Rt.__value = "PURExpr_DHFR_ctrl.fasta.txt"),
        (Rt.value = Rt.__value),
        r(Rt, "class", "svelte-134jeqo"),
        (Lt.__value = "SV40.fasta.txt"),
        (Lt.value = Lt.__value),
        r(Lt, "class", "svelte-134jeqo"),
        (Bl.__value = "YCplac111.fasta.txt"),
        (Bl.value = Bl.__value),
        r(Bl, "class", "svelte-134jeqo"),
        (bl.__value = "YCplac22.fasta.txt"),
        (bl.value = bl.__value),
        r(bl, "class", "svelte-134jeqo"),
        (Ut.__value = "YCplac33.fasta.txt"),
        (Ut.value = Ut.__value),
        r(Ut, "class", "svelte-134jeqo"),
        (il.__value = "YEp24.fasta.txt"),
        (il.value = il.__value),
        r(il, "class", "svelte-134jeqo"),
        (ll.__value = "YEplac112.fasta.txt"),
        (ll.value = ll.__value),
        r(ll, "class", "svelte-134jeqo"),
        (ql.__value = "YEplac181.fasta.txt"),
        (ql.value = ql.__value),
        r(ql, "class", "svelte-134jeqo"),
        (Hl.__value = "YEplac195.fasta.txt"),
        (Hl.value = Hl.__value),
        r(Hl, "class", "svelte-134jeqo"),
        (Ul.__value = "YIplac128.fasta.txt"),
        (Ul.value = Ul.__value),
        r(Ul, "class", "svelte-134jeqo"),
        (us.__value = "YIplac204.fasta.txt"),
        (us.value = us.__value),
        r(us, "class", "svelte-134jeqo"),
        (It.__value = "YIplac211.fasta.txt"),
        (It.value = It.__value),
        r(It, "class", "svelte-134jeqo"),
        r(p, "id", "samplevectors"),
        r(p, "aria-label", "Sample vectors"),
        r(p, "class", "dropdown neb-dropdown svelte-134jeqo"),
        l[5] === void 0 && vn(() => l[24].call(p)),
        r(d, "class", "col svelte-134jeqo"),
        r(ol, "class", "col svelte-134jeqo"),
        r(n, "class", "row svelte-134jeqo"),
        r(xt, "class", "svelte-134jeqo"),
        r(fs, "class", " svelte-134jeqo"),
        r(fs, "type", "checkbox"),
        r(fs, "id", "useRevComp"),
        r(es, "for", "useRevComp"),
        r(es, "class", "svelte-134jeqo"),
        r($l, "class", "svelte-134jeqo"),
        r(yl, "class", " svelte-134jeqo"),
        r(yl, "type", "checkbox"),
        r(yl, "id", "partialseq"),
        r(Vl, "for", "partialseq"),
        r(Vl, "class", "svelte-134jeqo"),
        r(ds, "class", "svelte-134jeqo"),
        r(ps, "class", "button btn btn-link neb-blue svelte-134jeqo"),
        r(ps, "id", "clearSeq"),
        r(Is, "class", "svelte-134jeqo"),
        r(wl, "class", "col svelte-134jeqo"),
        r(Fs, "class", "row svelte-134jeqo"),
        r(Cl, "id", "target"),
        r(Cl, "aria-label", "Sequence"),
        (Cl.value = l[1]),
        r(Cl, "type", "text"),
        r(Cl, "class", "longseqinput svelte-134jeqo"),
        r(Cl, "placeholder", "FASTA or raw sequence. ACGT and spaces accepted; no ambiguous bases allowed."),
        r(Ss, "class", "col svelte-134jeqo"),
        r(al, "class", "row svelte-134jeqo"),
        r(hs, "class", "col svelte-134jeqo"),
        r(Ds, "class", "row svelte-134jeqo"),
        r(Yt, "for", "seqid"),
        r(Yt, "class", "svelte-134jeqo"),
        r(ls, "type", "text"),
        r(ls, "id", "seqid"),
        r(ls, "class", " svelte-134jeqo"),
        r(js, "class", "col svelte-134jeqo"),
        r(Os, "class", "row svelte-134jeqo"),
        r(gs, "class", "row svelte-134jeqo"),
        r(Bs, "class", "row svelte-134jeqo"));
    },
    m(Ae, ot) {
      (Ge(e, Ae, ot),
        g(Ae, s, ot),
        g(Ae, n, ot),
        o(n, a),
        o(a, i),
        o(i, u),
        l[22](i),
        o(a, f),
        o(n, h),
        o(n, d),
        o(d, p),
        o(p, v),
        o(p, _),
        o(p, b),
        o(p, w),
        o(p, y),
        o(p, A),
        o(p, q),
        o(p, T),
        o(p, N),
        o(p, M),
        o(p, k),
        o(p, E),
        o(p, P),
        o(p, G),
        o(p, O),
        o(p, K),
        o(p, D),
        o(p, L),
        o(p, R),
        o(p, Q),
        o(p, ae),
        o(p, W),
        o(p, x),
        o(p, H),
        o(p, U),
        o(p, le),
        o(p, J),
        o(p, ie),
        o(p, z),
        o(p, te),
        o(p, pe),
        o(p, Y),
        o(p, ue),
        o(p, he),
        o(p, de),
        o(p, _e),
        o(p, ne),
        o(p, $),
        o(p, fe),
        o(p, we),
        o(p, j),
        o(p, F),
        o(p, ee),
        o(p, V),
        o(p, me),
        o(p, _t),
        o(p, Je),
        o(p, bt),
        o(p, ut),
        o(p, it),
        o(p, Be),
        o(p, We),
        o(p, Ie),
        o(p, Ce),
        o(p, De),
        o(p, at),
        o(p, Ve),
        o(p, $t),
        o(p, rt),
        o(p, Xt),
        o(p, Xe),
        o(p, Oe),
        o(p, He),
        o(p, Ke),
        o(p, ke),
        o(p, Ue),
        o(p, nl),
        o(p, At),
        o(p, Gt),
        o(p, pt),
        o(p, Ee),
        o(p, Qe),
        o(p, ht),
        o(p, tt),
        o(p, pl),
        o(p, Kl),
        o(p, Mt),
        o(p, hl),
        o(p, Et),
        o(p, bs),
        o(p, kl),
        o(p, qs),
        o(p, Bt),
        o(p, ws),
        o(p, Vt),
        o(p, Ft),
        o(p, Yl),
        o(p, Ol),
        o(p, Wl),
        o(p, ml),
        o(p, jl),
        o(p, Gl),
        o(p, Nt),
        o(p, zl),
        o(p, rl),
        o(p, rs),
        o(p, el),
        o(p, os),
        o(p, tl),
        o(p, qt),
        o(p, gl),
        o(p, Fl),
        o(p, Xl),
        o(p, Ze),
        o(p, vl),
        o(p, Ql),
        o(p, xl),
        o(p, lt),
        o(p, st),
        o(p, Ml),
        o(p, Kt),
        o(p, ys),
        o(p, Jl),
        o(p, as),
        o(p, Pt),
        o(p, Ht),
        o(p, Zl),
        o(p, cs),
        o(p, Qt),
        o(p, As),
        o(p, _l),
        o(p, Rt),
        o(p, Lt),
        o(p, Bl),
        o(p, bl),
        o(p, Ut),
        o(p, il),
        o(p, ll),
        o(p, ql),
        o(p, Hl),
        o(p, Ul),
        o(p, us),
        o(p, It),
        l[23](p),
        gn(p, l[5], !0),
        o(d, Ks),
        ul && ul.m(d, null),
        o(n, zs),
        o(n, ol),
        g(Ae, Gs, ot),
        g(Ae, xt, ot),
        g(Ae, tn, ot),
        g(Ae, Fs, ot),
        o(Fs, wl),
        o(wl, ln),
        o(wl, $l),
        o($l, fs),
        (fs.checked = l[2]),
        o($l, Rs),
        o($l, es),
        o(wl, Cs),
        o(wl, ds),
        o(ds, yl),
        (yl.checked = l[3]),
        o(ds, un),
        o(ds, Vl),
        o(ds, Ys),
        Ge(ts, ds, null),
        o(wl, Ts),
        o(wl, Is),
        o(Is, ps),
        o(wl, Al),
        Tl && Tl.m(wl, null),
        g(Ae, Xs, ot),
        El && El.m(Ae, ot),
        g(Ae, sl, ot),
        g(Ae, al, ot),
        o(al, Ss),
        o(Ss, Cl),
        l[29](Cl),
        g(Ae, ks, ot),
        g(Ae, Ds, ot),
        o(Ds, hs),
        o(hs, Qs),
        o(hs, xs),
        o(hs, Ms),
        g(Ae, sn, ot),
        g(Ae, Os, ot),
        o(Os, js),
        o(js, Yt),
        o(js, cl),
        o(js, ls),
        nt(ls, l[4]),
        g(Ae, nn, ot),
        g(Ae, gs, ot),
        g(Ae, rn, ot),
        g(Ae, Bs, ot),
        (Hs = !0),
        Js ||
          ((fn = [
            ce(u, "change", l[21]),
            ce(u, "change", l[16]),
            ce(p, "change", l[24]),
            ce(p, "change", l[25]),
            ce(fs, "change", l[26]),
            ce(yl, "change", l[27]),
            ce(ps, "click", l[18]),
            ce(Cl, "input", l[14]),
            ce(Cl, "blur", l[15]),
            ce(ls, "input", l[30]),
          ]),
          (Js = !0)));
    },
    p(Ae, ot) {
      const on = {};
      (ot[1] & 524288 && (on.$$scope = { dirty: ot, ctx: Ae }),
        !t && ot[0] & 4096 && ((t = !0), (on.dialog = Ae[12]), Wt(() => (t = !1))),
        e.$set(on),
        ot[0] & 32 && gn(p, Ae[5]),
        Ae[13] ? (ul ? ul.p(Ae, ot) : ((ul = Oi(Ae)), ul.c(), ul.m(d, null))) : ul && (ul.d(1), (ul = null)),
        ot[0] & 4 && (fs.checked = Ae[2]),
        ot[0] & 8 && (yl.checked = Ae[3]),
        Ae[9] ? Tl || ((Tl = ji()), Tl.c(), Tl.m(wl, null)) : Tl && (Tl.d(1), (Tl = null)),
        Ae[10] ? (El ? El.p(Ae, ot) : ((El = Bi(Ae)), El.c(), El.m(sl.parentNode, sl))) : El && (El.d(1), (El = null)),
        (!Hs || ot[0] & 2) && (Cl.value = Ae[1]),
        (!Hs || ot[0] & 1) && ms !== (ms = Ae[0].seq.length + "") && Z(xs, ms),
        ot[0] & 16 && ls.value !== Ae[4] && nt(ls, Ae[4]));
    },
    i(Ae) {
      Hs || (ve(e.$$.fragment, Ae), ve(ts.$$.fragment, Ae), (Hs = !0));
    },
    o(Ae) {
      (be(e.$$.fragment, Ae), be(ts.$$.fragment, Ae), (Hs = !1));
    },
    d(Ae) {
      (Fe(e, Ae),
        Ae && m(s),
        Ae && m(n),
        l[22](null),
        l[23](null),
        ul && ul.d(),
        Ae && m(Gs),
        Ae && m(xt),
        Ae && m(tn),
        Ae && m(Fs),
        Fe(ts),
        Tl && Tl.d(),
        Ae && m(Xs),
        El && El.d(Ae),
        Ae && m(sl),
        Ae && m(al),
        l[29](null),
        Ae && m(ks),
        Ae && m(Ds),
        Ae && m(sn),
        Ae && m(Os),
        Ae && m(nn),
        Ae && m(gs),
        Ae && m(rn),
        Ae && m(Bs),
        (Js = !1),
        Zt(fn));
    },
  };
}
const ca = 150;
function ua(l, e, t) {
  let {
    seqrec: s = {
      seq: "",
      id: "unknown",
      format: "unknown",
      header: "",
      isCircular: !0,
      useRevComp: !1,
      isVector: !0,
      type: "dna",
      len: 0,
      start: 0,
      end: 0,
      frag: null,
    },
  } = e;
  var n = "",
    a = !1,
    i = !1,
    u,
    f = "unselected",
    h,
    d,
    p = "",
    v = "INPUTSEQ",
    _ = !1,
    b = "",
    w = null,
    y,
    A = null,
    q = null,
    T = function (ne) {
      ((q = ne.target.value), clearTimeout(A), (A = setTimeout(N, ca)));
    },
    N = function () {
      (clearTimeout(A), (A = null), q !== null && (t(1, (n = q)), (q = null)));
    },
    M = function (ne) {
      (clearTimeout(A), (A = null), (q = null), t(1, (n = ne)), y && t(11, (y.value = ne), y));
    },
    k;
  const E = function () {
      (h && h.reset(), t(5, (f = "unselected")));
    },
    P = function (ne) {
      var fe = ne.target.result;
      (M(fe), (ne.target.value = ""));
    },
    G = function (ne) {},
    O = function (ne) {
      const $ = new FileReader();
      ($.readAsText(ne), $.addEventListener("loadend", P), $.addEventListener("error", G));
    };
  var K = function () {
      const ne = document.getElementById("fileinput").files[0];
      ne && O(ne);
    },
    D = null,
    L = "",
    R = async function (ne) {
      if (!ne || ne === "unselected") return;
      D && D.abort();
      const $ = new AbortController();
      ((D = $), t(13, (L = "")));
      try {
        const fe = await fetch("/seqs/" + ne, { signal: $.signal });
        if (!fe.ok) throw new Error("the server answered " + fe.status);
        const we = await fe.text();
        if (/^\s*</.test(we)) throw new Error("not a sequence file");
        $ === D && M(we);
      } catch (fe) {
        if (fe && fe.name === "AbortError") return;
        (t(
          13,
          (L =
            "Could not load the sample vector " +
            ne +
            " (" +
            (fe && fe.message ? fe.message : fe) +
            "). The current sequence was left unchanged."),
        ),
          t(5, (f = "unselected")));
      } finally {
        D === $ && (D = null);
      }
    },
    Q = function (ne) {
      t(13, (L = ""));
    },
    ae = function (ne, $) {
      var fe = [],
        we = !1;
      return (
        ne && ne.recordCount > 1 && fe.push("using the first of " + ne.recordCount + " records (" + ne.id + ")"),
        ne &&
          ne.format === "genbank" &&
          ne.topology === "linear" &&
          ($ !== w && ((w = $), i || (t(3, (i = !0)), (we = !0))),
          (i || we) && fe.push("GenBank record is linear: treating as a partial sequence")),
        t(10, (b = fe.join("; "))),
        we
      );
    },
    W = function (ne, $, fe, we) {
      if ((we || (we = !1), !ne || !$))
        try {
          ne = new zo.Fragment("", { srctopology: we ? "linear" : "circular", srctype: "synth" });
        } catch {
          return null;
        }
      var j,
        F = !1;
      try {
        j = Xo.parseSeqFile($, null, null);
      } catch {
        ((j = { id: "", seq: "" }), (F = !0));
      }
      var ee = !1;
      try {
        ee = ae(j, $);
      } catch {}
      try {
        (ne.updateSrcseq(j.seq), t(9, (_ = F)));
      } catch {
        (ne.updateSrcseq(""), t(9, (_ = !0)));
      }
      return (
        fe && ne.updateSrcseq(Te.revcomp(ne.srcseq.wseq)),
        (ne.id = j.id),
        (ne.isCircular = !(we || ee)),
        t(0, (s.frag = ne), s),
        t(0, (s.seq = j.seq), s),
        t(0, (s.len = j.len), s),
        t(0, (s.id = j.id), s),
        t(4, (v = s.id)),
        E(),
        ne
      );
    },
    x = function () {
      (M(""),
        t(
          0,
          (s = {
            seq: "",
            id: "unknown",
            format: "unknown",
            header: "",
            isCircular: !0,
            useRevComp: !1,
            isVector: !0,
            type: "dna",
            len: 0,
            start: 0,
            end: 0,
            frag: null,
          }),
        ),
        t(3, (i = !1)),
        t(2, (a = !1)),
        t(10, (b = "")),
        (w = null));
    },
    H = function (ne) {
      return (!ne && !s.id && (ne = "INPUTSEQ_" + s.len + "bp"), ne !== s.id && (t(0, (s.id = ne), s), t(19, (u.id = s.id), u)), ne);
    };
  const U = go("seqinput");
  (U && ({ rawseq: n, useRevComp: a, partialseq: i, seqid: v, autoPartialAppliedFor: w } = U),
    Dn(() => {
      (N(), mo("seqinput", { rawseq: n, useRevComp: a, partialseq: i, seqid: v, autoPartialAppliedFor: w }));
    }));
  function le(ne) {
    ((k = ne), t(12, k));
  }
  function J() {
    ((p = this.value), t(8, p));
  }
  function ie(ne) {
    dt[ne ? "unshift" : "push"](() => {
      ((h = ne), t(6, h));
    });
  }
  function z(ne) {
    dt[ne ? "unshift" : "push"](() => {
      ((d = ne), t(7, d));
    });
  }
  function te() {
    ((f = Gn(this)), t(5, f));
  }
  const pe = () => R(f);
  function Y() {
    ((a = this.checked), t(2, a));
  }
  function ue() {
    ((i = this.checked), t(3, i));
  }
  const he = () => k.showModal();
  function de(ne) {
    dt[ne ? "unshift" : "push"](() => {
      ((y = ne), t(11, y));
    });
  }
  function _e() {
    ((v = this.value), t(4, v), t(48, H));
  }
  return (
    (l.$$set = (ne) => {
      "seqrec" in ne && t(0, (s = ne.seqrec));
    }),
    (l.$$.update = () => {
      if ((l.$$.dirty[0] & 524302 && t(19, (u = W(u, n, a, i))), l.$$.dirty[0] & 2 && Q(), l.$$.dirty[0] & 524296 && u))
        try {
          u.srcseq.setSeqTopology(i ? "linear" : "circular");
        } catch {}
      (l.$$.dirty[0] & 4 && t(0, (s.useRevComp = a), s),
        l.$$.dirty[0] & 524288 && t(0, (s.frag = u), s),
        l.$$.dirty[0] & 524288 && t(0, (s.seq = u ? u.srcseq.wseq : ""), s),
        l.$$.dirty[0] & 8 && t(0, (s.isCircular = !i), s),
        l.$$.dirty[0] & 16 && t(4, (v = H(v))),
        l.$$.dirty[0] & 16 && t(0, (s.id = v), s));
    }),
    [s, n, a, i, v, f, h, d, p, _, b, y, k, L, T, N, K, R, x, u, le, J, ie, z, te, pe, Y, ue, he, de, _e]
  );
}
class fa extends Dl {
  constructor(e) {
    (super(), Il(this, e, ua, aa, Ll, { seqrec: 0 }, null, [-1, -1]));
  }
}
function Hi(l, e, t) {
  const s = l.slice();
  return ((s[4] = e[t]), (s[6] = t), s);
}
function Ui(l) {
  let e,
    t,
    s,
    n,
    a,
    i,
    u,
    f = l[1],
    h = [];
  for (let d = 0; d < f.length; d += 1) h[d] = Vi(Hi(l, f, d));
  return {
    c() {
      ((e = c("div")),
        (t = c("table")),
        (s = c("thead")),
        (s.innerHTML = `<tr class="svelte-oeg76i"><th class="svelte-oeg76i">View</th> 
				<th class="svelte-oeg76i">Description</th> 
	        	<th class="svelte-oeg76i">ID (F/R)</th> 
	            <th class="svelte-oeg76i">Oligo (lowercase = mutation)</th> 
	            <th class="svelte-oeg76i">Len</th> 
	            <th class="svelte-oeg76i">Tm</th> 
	            <th class="svelte-oeg76i">Ta *</th></tr>`),
        (n = S()),
        (a = c("tbody")));
      for (let d = 0; d < h.length; d += 1) h[d].c();
      ((i = S()),
        (u = c("div")),
        (u.innerHTML = "* <small>Annealing temperature</small>"),
        r(s, "class", "svelte-oeg76i"),
        r(t, "id", "oligotable"),
        r(t, "class", " svelte-oeg76i"));
    },
    m(d, p) {
      (g(d, e, p), o(e, t), o(t, s), o(t, n), o(t, a));
      for (let v = 0; v < h.length; v += 1) h[v] && h[v].m(a, null);
      (o(e, i), o(e, u));
    },
    p(d, p) {
      if (p & 3) {
        f = d[1];
        let v;
        for (v = 0; v < f.length; v += 1) {
          const _ = Hi(d, f, v);
          h[v] ? h[v].p(_, p) : ((h[v] = Vi(_)), h[v].c(), h[v].m(a, null));
        }
        for (; v < h.length; v += 1) h[v].d(1);
        h.length = f.length;
      }
    },
    d(d) {
      (d && m(e), cn(h, d));
    },
  };
}
function Vi(l) {
  let e,
    t,
    s,
    n,
    a,
    i = l[4].description + "",
    u,
    f,
    h,
    d = Jt.primerId(l[4], "fwd") + "",
    p,
    v,
    _,
    b = Jt.primerSeq(l[4], "fwd") + "",
    w,
    y,
    A,
    q = l[4].lenf + "",
    T,
    N,
    M,
    k = Math.round(l[4].tmf) + "",
    E,
    P,
    G,
    O = Math.round(l[4].ta) + "",
    K,
    D,
    L,
    R,
    Q = Jt.primerId(l[4], "rev") + "",
    ae,
    W,
    x,
    H = Jt.primerSeq(l[4], "rev") + "",
    U,
    le,
    J,
    ie = l[4].lenr + "",
    z,
    te,
    pe,
    Y = Math.round(l[4].tmr) + "",
    ue,
    he,
    de,
    _e,
    ne;
  return (
    (de = Ti(l[3][0])),
    {
      c() {
        ((e = c("tr")),
          (t = c("td")),
          (s = c("input")),
          (n = S()),
          (a = c("td")),
          (u = C(i)),
          (f = S()),
          (h = c("td")),
          (p = C(d)),
          (v = S()),
          (_ = c("td")),
          (w = C(b)),
          (y = S()),
          (A = c("td")),
          (T = C(q)),
          (N = S()),
          (M = c("td")),
          (E = C(k)),
          (P = S()),
          (G = c("td")),
          (K = C(O)),
          (D = S()),
          (L = c("tr")),
          (R = c("td")),
          (ae = C(Q)),
          (W = S()),
          (x = c("td")),
          (U = C(H)),
          (le = S()),
          (J = c("td")),
          (z = C(ie)),
          (te = S()),
          (pe = c("td")),
          (ue = C(Y)),
          (he = S()),
          r(s, "type", "radio"),
          r(s, "name", "vwtxl"),
          (s.__value = l[6]),
          (s.value = s.__value),
          r(s, "aria-label", "Select pair " + (l[6] + 1)),
          r(t, "rowspan", "2"),
          r(t, "class", "svelte-oeg76i"),
          r(a, "rowspan", "2"),
          r(a, "class", "tblcoldesc svelte-oeg76i"),
          r(h, "class", "svelte-oeg76i"),
          r(_, "class", "tblcolseq f1b svelte-oeg76i"),
          r(A, "class", "svelte-oeg76i"),
          r(M, "class", "svelte-oeg76i"),
          r(G, "rowspan", "2"),
          r(G, "class", "tblta svelte-oeg76i"),
          r(e, "class", "svelte-oeg76i"),
          r(R, "class", "svelte-oeg76i"),
          r(x, "class", "tblcolseq f3b svelte-oeg76i"),
          r(J, "class", "svelte-oeg76i"),
          r(pe, "class", "svelte-oeg76i"),
          r(L, "class", "svelte-oeg76i"),
          de.p(s));
      },
      m($, fe) {
        (g($, e, fe),
          o(e, t),
          o(t, s),
          (s.checked = s.__value === l[0]),
          o(e, n),
          o(e, a),
          o(a, u),
          o(e, f),
          o(e, h),
          o(h, p),
          o(e, v),
          o(e, _),
          o(_, w),
          o(e, y),
          o(e, A),
          o(A, T),
          o(e, N),
          o(e, M),
          o(M, E),
          o(e, P),
          o(e, G),
          o(G, K),
          g($, D, fe),
          g($, L, fe),
          o(L, R),
          o(R, ae),
          o(L, W),
          o(L, x),
          o(x, U),
          o(L, le),
          o(L, J),
          o(J, z),
          o(L, te),
          o(L, pe),
          o(pe, ue),
          o(L, he),
          _e || ((ne = ce(s, "change", l[2])), (_e = !0)));
      },
      p($, fe) {
        (fe & 3 && (s.checked = s.__value === $[0]),
          fe & 2 && i !== (i = $[4].description + "") && Z(u, i),
          fe & 2 && d !== (d = Jt.primerId($[4], "fwd") + "") && Z(p, d),
          fe & 2 && b !== (b = Jt.primerSeq($[4], "fwd") + "") && Z(w, b),
          fe & 2 && q !== (q = $[4].lenf + "") && Z(T, q),
          fe & 2 && k !== (k = Math.round($[4].tmf) + "") && Z(E, k),
          fe & 2 && O !== (O = Math.round($[4].ta) + "") && Z(K, O),
          fe & 2 && Q !== (Q = Jt.primerId($[4], "rev") + "") && Z(ae, Q),
          fe & 2 && H !== (H = Jt.primerSeq($[4], "rev") + "") && Z(U, H),
          fe & 2 && ie !== (ie = $[4].lenr + "") && Z(z, ie),
          fe & 2 && Y !== (Y = Math.round($[4].tmr) + "") && Z(ue, Y));
      },
      d($) {
        ($ && m(e), $ && m(D), $ && m(L), de.r(), (_e = !1), ne());
      },
    }
  );
}
function da(l) {
  let e,
    t = l[1].length > 0 && Ui(l);
  return {
    c() {
      (t && t.c(), (e = is()));
    },
    m(s, n) {
      (t && t.m(s, n), g(s, e, n));
    },
    p(s, [n]) {
      s[1].length > 0 ? (t ? t.p(s, n) : ((t = Ui(s)), t.c(), t.m(e.parentNode, e))) : t && (t.d(1), (t = null));
    },
    i: qe,
    o: qe,
    d(s) {
      (t && t.d(s), s && m(e));
    },
  };
}
function pa(l, e, t) {
  let { primers: s = [] } = e,
    { selectedtxl: n = 0 } = e;
  const a = [[]];
  function i() {
    ((n = this.__value), t(0, n));
  }
  return (
    (l.$$set = (u) => {
      ("primers" in u && t(1, (s = u.primers)), "selectedtxl" in u && t(0, (n = u.selectedtxl)));
    }),
    [n, s, i, a]
  );
}
class ha extends Dl {
  constructor(e) {
    (super(), Il(this, e, pa, da, Ll, { primers: 1, selectedtxl: 0 }));
  }
}
function Ki(l, e, t) {
  const s = l.slice();
  return ((s[2] = e[t]), (s[4] = t), s);
}
function Yi(l) {
  let e,
    t,
    s,
    n,
    a,
    i,
    u,
    f,
    h,
    d,
    p,
    v = l[0],
    _ = [];
  for (let b = 0; b < v.length; b += 1) _[b] = Xi(Ki(l, v, b));
  return {
    c() {
      ((e = c("div")),
        (t = c("table")),
        (s = c("thead")),
        (s.innerHTML = `<tr class="svelte-1gr9q7o"><th class="svelte-1gr9q7o"></th> 
				<th class="svelte-1gr9q7o">Description</th> 
	        	<th class="svelte-1gr9q7o">ID (F/R)</th> 
	            <th class="svelte-1gr9q7o">Oligo (lowercase = mutation)</th> 
	            <th class="svelte-1gr9q7o">Len</th> 
	            <th class="svelte-1gr9q7o">Tm</th> 
	            <th class="svelte-1gr9q7o">Ta \xB0C*</th> 
	            <th class="svelte-1gr9q7o">PCR Frag Len (bp)</th></tr>`),
        (n = S()),
        (a = c("tbody")));
      for (let b = 0; b < _.length; b += 1) _[b].c();
      ((i = S()),
        (u = c("div")),
        (u.innerHTML = "* <small>Annealing temperature</small>"),
        (f = S()),
        (h = c("div")),
        (h.innerHTML =
          "<small><span>\u21A9</span> Length of fragment that spans sequence origin assumes complete sequence was entered.</small>"),
        (d = S()),
        (p = c("div")),
        (p.innerHTML = "<small><span>\u21B9</span> Fragment length is very short.  Primers may overlap.  NOT RECOMMENDED.</small>"),
        r(s, "class", "svelte-1gr9q7o"),
        r(t, "id", "oligotable"),
        r(t, "class", " svelte-1gr9q7o"));
    },
    m(b, w) {
      (g(b, e, w), o(e, t), o(t, s), o(t, n), o(t, a));
      for (let y = 0; y < _.length; y += 1) _[y] && _[y].m(a, null);
      (o(e, i), o(e, u), o(e, f), o(e, h), o(e, d), o(e, p));
    },
    p(b, w) {
      if (w & 1) {
        v = b[0];
        let y;
        for (y = 0; y < v.length; y += 1) {
          const A = Ki(b, v, y);
          _[y] ? _[y].p(A, w) : ((_[y] = Xi(A)), _[y].c(), _[y].m(a, null));
        }
        for (; y < _.length; y += 1) _[y].d(1);
        _.length = v.length;
      }
    },
    d(b) {
      (b && m(e), cn(_, b));
    },
  };
}
function Wi(l) {
  let e;
  return {
    c() {
      ((e = c("span")), (e.textContent = "\u21A9"));
    },
    m(t, s) {
      g(t, e, s);
    },
    d(t) {
      t && m(e);
    },
  };
}
function zi(l) {
  let e;
  return {
    c() {
      ((e = c("span")), (e.textContent = "\u21B9"), r(e, "class", "warn svelte-1gr9q7o"));
    },
    m(t, s) {
      g(t, e, s);
    },
    d(t) {
      t && m(e);
    },
  };
}
function Xi(l) {
  let e,
    t,
    s,
    n,
    a = l[2].description + "",
    i,
    u,
    f,
    h = Jt.primerId(l[2], "fwd") + "",
    d,
    p,
    v,
    _ = Jt.primerSeq(l[2], "fwd") + "",
    b,
    w,
    y,
    A = l[2].lenf + "",
    q,
    T,
    N,
    M = Math.round(l[2].tmf) + "",
    k,
    E,
    P,
    G = Math.round(l[2].ta) + "",
    O,
    K,
    D,
    L = l[2].fraglen + "",
    R,
    Q,
    ae,
    W,
    x,
    H,
    U,
    le = Jt.primerId(l[2], "rev") + "",
    J,
    ie,
    z,
    te = Jt.primerSeq(l[2], "rev") + "",
    pe,
    Y,
    ue,
    he = l[2].lenr + "",
    de,
    _e,
    ne,
    $ = Math.round(l[2].tmr) + "",
    fe,
    we,
    j = l[2].wraparound && Wi(),
    F = l[2].clash && zi();
  return {
    c() {
      ((e = c("tr")),
        (t = c("td")),
        (s = S()),
        (n = c("td")),
        (i = C(a)),
        (u = S()),
        (f = c("td")),
        (d = C(h)),
        (p = S()),
        (v = c("td")),
        (b = C(_)),
        (w = S()),
        (y = c("td")),
        (q = C(A)),
        (T = S()),
        (N = c("td")),
        (k = C(M)),
        (E = S()),
        (P = c("td")),
        (O = C(G)),
        (K = S()),
        (D = c("td")),
        (R = C(L)),
        (Q = S()),
        j && j.c(),
        (ae = S()),
        F && F.c(),
        (x = S()),
        (H = c("tr")),
        (U = c("td")),
        (J = C(le)),
        (ie = S()),
        (z = c("td")),
        (pe = C(te)),
        (Y = S()),
        (ue = c("td")),
        (de = C(he)),
        (_e = S()),
        (ne = c("td")),
        (fe = C($)),
        (we = S()),
        r(t, "rowspan", "2"),
        r(t, "class", "svelte-1gr9q7o"),
        r(n, "rowspan", "2"),
        r(n, "class", "tblcoldesc svelte-1gr9q7o"),
        r(f, "class", "svelte-1gr9q7o"),
        r(v, "class", "tblcolseq f1b svelte-1gr9q7o"),
        r(y, "class", "svelte-1gr9q7o"),
        r(N, "class", "svelte-1gr9q7o"),
        r(P, "rowspan", "2"),
        r(P, "class", "tblta svelte-1gr9q7o"),
        r(D, "rowspan", "2"),
        r(D, "class", (W = "tblta " + (l[2].clash ? "invalidinput" : "") + " svelte-1gr9q7o")),
        r(e, "class", "svelte-1gr9q7o"),
        r(U, "class", "svelte-1gr9q7o"),
        r(z, "class", "tblcolseq f3b svelte-1gr9q7o"),
        r(ue, "class", "svelte-1gr9q7o"),
        r(ne, "class", "svelte-1gr9q7o"),
        r(H, "class", "svelte-1gr9q7o"));
    },
    m(ee, V) {
      (g(ee, e, V),
        o(e, t),
        o(e, s),
        o(e, n),
        o(n, i),
        o(e, u),
        o(e, f),
        o(f, d),
        o(e, p),
        o(e, v),
        o(v, b),
        o(e, w),
        o(e, y),
        o(y, q),
        o(e, T),
        o(e, N),
        o(N, k),
        o(e, E),
        o(e, P),
        o(P, O),
        o(e, K),
        o(e, D),
        o(D, R),
        o(D, Q),
        j && j.m(D, null),
        o(D, ae),
        F && F.m(D, null),
        g(ee, x, V),
        g(ee, H, V),
        o(H, U),
        o(U, J),
        o(H, ie),
        o(H, z),
        o(z, pe),
        o(H, Y),
        o(H, ue),
        o(ue, de),
        o(H, _e),
        o(H, ne),
        o(ne, fe),
        o(H, we));
    },
    p(ee, V) {
      (V & 1 && a !== (a = ee[2].description + "") && Z(i, a),
        V & 1 && h !== (h = Jt.primerId(ee[2], "fwd") + "") && Z(d, h),
        V & 1 && _ !== (_ = Jt.primerSeq(ee[2], "fwd") + "") && Z(b, _),
        V & 1 && A !== (A = ee[2].lenf + "") && Z(q, A),
        V & 1 && M !== (M = Math.round(ee[2].tmf) + "") && Z(k, M),
        V & 1 && G !== (G = Math.round(ee[2].ta) + "") && Z(O, G),
        V & 1 && L !== (L = ee[2].fraglen + "") && Z(R, L),
        ee[2].wraparound ? j || ((j = Wi()), j.c(), j.m(D, ae)) : j && (j.d(1), (j = null)),
        ee[2].clash ? F || ((F = zi()), F.c(), F.m(D, null)) : F && (F.d(1), (F = null)),
        V & 1 && W !== (W = "tblta " + (ee[2].clash ? "invalidinput" : "") + " svelte-1gr9q7o") && r(D, "class", W),
        V & 1 && le !== (le = Jt.primerId(ee[2], "rev") + "") && Z(J, le),
        V & 1 && te !== (te = Jt.primerSeq(ee[2], "rev") + "") && Z(pe, te),
        V & 1 && he !== (he = ee[2].lenr + "") && Z(de, he),
        V & 1 && $ !== ($ = Math.round(ee[2].tmr) + "") && Z(fe, $));
    },
    d(ee) {
      (ee && m(e), j && j.d(), F && F.d(), ee && m(x), ee && m(H));
    },
  };
}
function ma(l) {
  let e,
    t = l[0].length > 0 && Yi(l);
  return {
    c() {
      (t && t.c(), (e = is()));
    },
    m(s, n) {
      (t && t.m(s, n), g(s, e, n));
    },
    p(s, [n]) {
      s[0].length > 0 ? (t ? t.p(s, n) : ((t = Yi(s)), t.c(), t.m(e.parentNode, e))) : t && (t.d(1), (t = null));
    },
    i: qe,
    o: qe,
    d(s) {
      (t && t.d(s), s && m(e));
    },
  };
}
function ga(l, e, t) {
  let { primers: s = [] } = e,
    { selectedtxl: n = 0 } = e;
  return (
    (l.$$set = (a) => {
      ("primers" in a && t(0, (s = a.primers)), "selectedtxl" in a && t(1, (n = a.selectedtxl)));
    }),
    [s, n]
  );
}
class va extends Dl {
  constructor(e) {
    (super(), Il(this, e, ga, ma, Ll, { primers: 0, selectedtxl: 1 }));
  }
}
function Qi(l) {
  let e,
    t,
    s,
    n,
    a = l[7][0] + "",
    i,
    u,
    f,
    h,
    d = l[7][1] + "",
    p,
    v,
    _,
    b,
    w = l[7][2] + "",
    y,
    A,
    q,
    T,
    N,
    M,
    k,
    E,
    P,
    G,
    O,
    K,
    D = l[7][5] + "",
    L,
    R,
    Q = l[7][6] + "",
    ae,
    W,
    x = l[7][7] + "",
    H,
    U;
  function le(Y, ue) {
    return Y[1] === "replace" && Y[5].pflap.length === 0 ? ba : _a;
  }
  let J = le(l),
    ie = J(l);
  function z(Y, ue) {
    return Y[1] === "replace" && Y[5].pflap.length === 0 ? wa : qa;
  }
  let te = z(l),
    pe = te(l);
  return {
    c() {
      ((e = c("div")),
        (t = c("pre")),
        (s = C("")),
        (n = c("span")),
        (i = C(a)),
        (f = C(`
`)),
        (h = c("span")),
        (p = C(d)),
        (_ = C(`
`)),
        (b = c("span")),
        (y = C(w)),
        (q = C(`
`)),
        (T = S()),
        (N = c("div")),
        (M = c("pre")),
        (k = C("")),
        ie.c(),
        (E = C(`
`)),
        pe.c(),
        (P = C(`
`)),
        (G = S()),
        (O = c("pre")),
        (K = C("")),
        (L = C(D)),
        (R = C(`
`)),
        (ae = C(Q)),
        (W = C(`
`)),
        (H = C(x)),
        (U = C(`

`)),
        r(n, "class", (u = Vs(l[4] == 3 ? "yellowbg" : "") + " svelte-chgkuk")),
        r(h, "class", (v = Vs(l[4] == 2 ? "yellowbg" : "") + " svelte-chgkuk")),
        r(b, "class", (A = Vs(l[4] == 1 ? "yellowbg" : "") + " svelte-chgkuk")),
        r(t, "class", "svelte-chgkuk"),
        r(M, "class", "svelte-chgkuk"),
        r(N, "class", "dsseqbox svelte-chgkuk"),
        r(O, "class", "svelte-chgkuk"),
        r(e, "id", "translation"),
        r(e, "class", "svelte-chgkuk"));
    },
    m(Y, ue) {
      (g(Y, e, ue),
        o(e, t),
        o(t, s),
        o(t, n),
        o(n, i),
        o(t, f),
        o(t, h),
        o(h, p),
        o(t, _),
        o(t, b),
        o(b, y),
        o(t, q),
        o(e, T),
        o(e, N),
        o(N, M),
        o(M, k),
        ie.m(M, null),
        o(M, E),
        pe.m(M, null),
        o(M, P),
        o(e, G),
        o(e, O),
        o(O, K),
        o(O, L),
        o(O, R),
        o(O, ae),
        o(O, W),
        o(O, H),
        o(O, U));
    },
    p(Y, ue) {
      (ue & 128 && a !== (a = Y[7][0] + "") && Z(i, a),
        ue & 16 && u !== (u = Vs(Y[4] == 3 ? "yellowbg" : "") + " svelte-chgkuk") && r(n, "class", u),
        ue & 128 && d !== (d = Y[7][1] + "") && Z(p, d),
        ue & 16 && v !== (v = Vs(Y[4] == 2 ? "yellowbg" : "") + " svelte-chgkuk") && r(h, "class", v),
        ue & 128 && w !== (w = Y[7][2] + "") && Z(y, w),
        ue & 16 && A !== (A = Vs(Y[4] == 1 ? "yellowbg" : "") + " svelte-chgkuk") && r(b, "class", A),
        J === (J = le(Y)) && ie ? ie.p(Y, ue) : (ie.d(1), (ie = J(Y)), ie && (ie.c(), ie.m(M, E))),
        te === (te = z(Y)) && pe ? pe.p(Y, ue) : (pe.d(1), (pe = te(Y)), pe && (pe.c(), pe.m(M, P))),
        ue & 128 && D !== (D = Y[7][5] + "") && Z(L, D),
        ue & 128 && Q !== (Q = Y[7][6] + "") && Z(ae, Q),
        ue & 128 && x !== (x = Y[7][7] + "") && Z(H, x));
    },
    d(Y) {
      (Y && m(e), ie.d(), pe.d());
    },
  };
}
function _a(l) {
  let e,
    t = Te.revcomp(l[6].primer).toUpperCase() + "",
    s,
    n,
    a = Te.revcomp(l[6].pflap).toLowerCase() + "",
    i,
    u,
    f = l[5].pflap.toLowerCase() + "",
    h,
    d,
    p = l[5].primer.toUpperCase() + "",
    v;
  return {
    c() {
      ((e = c("span")),
        (s = C(t)),
        (n = c("span")),
        (i = C(a)),
        (u = c("span")),
        (h = C(f)),
        (d = c("span")),
        (v = C(p)),
        r(e, "class", "primer oligo f0b svelte-chgkuk"),
        r(n, "class", "pflap oligo f0b svelte-chgkuk"),
        r(u, "class", "pflap oligo f1b svelte-chgkuk"),
        r(d, "class", "primer oligo f1b svelte-chgkuk"));
    },
    m(_, b) {
      (g(_, e, b), o(e, s), g(_, n, b), o(n, i), g(_, u, b), o(u, h), g(_, d, b), o(d, v));
    },
    p(_, b) {
      (b & 64 && t !== (t = Te.revcomp(_[6].primer).toUpperCase() + "") && Z(s, t),
        b & 64 && a !== (a = Te.revcomp(_[6].pflap).toLowerCase() + "") && Z(i, a),
        b & 32 && f !== (f = _[5].pflap.toLowerCase() + "") && Z(h, f),
        b & 32 && p !== (p = _[5].primer.toUpperCase() + "") && Z(v, p));
    },
    d(_) {
      (_ && m(e), _ && m(n), _ && m(u), _ && m(d));
    },
  };
}
function ba(l) {
  let e,
    t = Te.revcomp(l[6].primer).toUpperCase() + "",
    s,
    n,
    a = Te.revcomp(l[6].pflap).toLowerCase() + "",
    i,
    u,
    f = l[5].pflap.toLowerCase() + "",
    h,
    d,
    p = l[5].primer.slice(0, l[3]).toUpperCase() + "",
    v,
    _,
    b = l[5].primer.slice(l[3], l[3] + l[2].length).toLowerCase() + "",
    w,
    y,
    A = l[5].primer.slice(l[3] + l[2].length).toUpperCase() + "",
    q;
  return {
    c() {
      ((e = c("span")),
        (s = C(t)),
        (n = c("span")),
        (i = C(a)),
        (u = c("span")),
        (h = C(f)),
        (d = c("span")),
        (v = C(p)),
        (_ = c("span")),
        (w = C(b)),
        (y = c("span")),
        (q = C(A)),
        r(e, "class", "primer oligo f0b svelte-chgkuk"),
        r(n, "class", "pflap oligo f0b svelte-chgkuk"),
        r(u, "class", "pflap oligo f1b svelte-chgkuk"),
        r(d, "class", "primer oligo f1b svelte-chgkuk"),
        r(_, "class", "primer oligo pflap f1b svelte-chgkuk"),
        r(y, "class", "primer oligo f1b svelte-chgkuk"));
    },
    m(T, N) {
      (g(T, e, N), o(e, s), g(T, n, N), o(n, i), g(T, u, N), o(u, h), g(T, d, N), o(d, v), g(T, _, N), o(_, w), g(T, y, N), o(y, q));
    },
    p(T, N) {
      (N & 64 && t !== (t = Te.revcomp(T[6].primer).toUpperCase() + "") && Z(s, t),
        N & 64 && a !== (a = Te.revcomp(T[6].pflap).toLowerCase() + "") && Z(i, a),
        N & 32 && f !== (f = T[5].pflap.toLowerCase() + "") && Z(h, f),
        N & 40 && p !== (p = T[5].primer.slice(0, T[3]).toUpperCase() + "") && Z(v, p),
        N & 44 && b !== (b = T[5].primer.slice(T[3], T[3] + T[2].length).toLowerCase() + "") && Z(w, b),
        N & 44 && A !== (A = T[5].primer.slice(T[3] + T[2].length).toUpperCase() + "") && Z(q, A));
    },
    d(T) {
      (T && m(e), T && m(n), T && m(u), T && m(d), T && m(_), T && m(y));
    },
  };
}
function qa(l) {
  let e,
    t = Te.rev(l[6].primer).toUpperCase() + "",
    s,
    n,
    a = Te.rev(l[6].pflap).toLowerCase() + "",
    i,
    u,
    f = Te.comp(l[5].pflap).toLowerCase() + "",
    h,
    d,
    p = Te.comp(l[5].primer).toUpperCase() + "",
    v;
  return {
    c() {
      ((e = c("span")),
        (s = C(t)),
        (n = c("span")),
        (i = C(a)),
        (u = c("span")),
        (h = C(f)),
        (d = c("span")),
        (v = C(p)),
        r(e, "class", "primer oligo f3b svelte-chgkuk"),
        r(n, "class", "pflap oligo f3b svelte-chgkuk"),
        r(u, "class", "pflap oligo f0b svelte-chgkuk"),
        r(d, "class", "primer oligo f0b svelte-chgkuk"));
    },
    m(_, b) {
      (g(_, e, b), o(e, s), g(_, n, b), o(n, i), g(_, u, b), o(u, h), g(_, d, b), o(d, v));
    },
    p(_, b) {
      (b & 64 && t !== (t = Te.rev(_[6].primer).toUpperCase() + "") && Z(s, t),
        b & 64 && a !== (a = Te.rev(_[6].pflap).toLowerCase() + "") && Z(i, a),
        b & 32 && f !== (f = Te.comp(_[5].pflap).toLowerCase() + "") && Z(h, f),
        b & 32 && p !== (p = Te.comp(_[5].primer).toUpperCase() + "") && Z(v, p));
    },
    d(_) {
      (_ && m(e), _ && m(n), _ && m(u), _ && m(d));
    },
  };
}
function wa(l) {
  let e,
    t = Te.rev(l[6].primer).toUpperCase() + "",
    s,
    n,
    a = Te.rev(l[6].pflap).toLowerCase() + "",
    i,
    u,
    f = Te.comp(l[5].primer).slice(0, l[3]).toUpperCase() + "",
    h,
    d,
    p =
      Te.comp(l[5].primer)
        .slice(l[3], l[3] + l[2].length)
        .toLowerCase() + "",
    v,
    _,
    b =
      Te.comp(l[5].primer)
        .slice(l[3] + l[2].length)
        .toUpperCase() + "",
    w;
  return {
    c() {
      ((e = c("span")),
        (s = C(t)),
        (n = c("span")),
        (i = C(a)),
        (u = c("span")),
        (h = C(f)),
        (d = c("span")),
        (v = C(p)),
        (_ = c("span")),
        (w = C(b)),
        r(e, "class", "primer oligo f3b svelte-chgkuk"),
        r(n, "class", "pflap oligo f3b svelte-chgkuk"),
        r(u, "class", "primer oligo f0b svelte-chgkuk"),
        r(d, "class", "primer oligo pflap f0b svelte-chgkuk"),
        r(_, "class", "primer oligo f0b svelte-chgkuk"));
    },
    m(y, A) {
      (g(y, e, A), o(e, s), g(y, n, A), o(n, i), g(y, u, A), o(u, h), g(y, d, A), o(d, v), g(y, _, A), o(_, w));
    },
    p(y, A) {
      (A & 64 && t !== (t = Te.rev(y[6].primer).toUpperCase() + "") && Z(s, t),
        A & 64 && a !== (a = Te.rev(y[6].pflap).toLowerCase() + "") && Z(i, a),
        A & 40 && f !== (f = Te.comp(y[5].primer).slice(0, y[3]).toUpperCase() + "") && Z(h, f),
        A & 44 &&
          p !==
            (p =
              Te.comp(y[5].primer)
                .slice(y[3], y[3] + y[2].length)
                .toLowerCase() + "") &&
          Z(v, p),
        A & 44 &&
          b !==
            (b =
              Te.comp(y[5].primer)
                .slice(y[3] + y[2].length)
                .toUpperCase() + "") &&
          Z(w, b));
    },
    d(y) {
      (y && m(e), y && m(n), y && m(u), y && m(d), y && m(_));
    },
  };
}
function ya(l) {
  let e,
    t = l[0] && Qi(l);
  return {
    c() {
      (t && t.c(), (e = is()));
    },
    m(s, n) {
      (t && t.m(s, n), g(s, e, n));
    },
    p(s, [n]) {
      s[0] ? (t ? t.p(s, n) : ((t = Qi(s)), t.c(), t.m(e.parentNode, e))) : t && (t.d(1), (t = null));
    },
    i: qe,
    o: qe,
    d(s) {
      (t && t.d(s), s && m(e));
    },
  };
}
function Aa(l, e, t) {
  let { primerpair: s } = e,
    { seqop: n } = e,
    { repseq: a = "" } = e,
    { pad: i = 10 } = e,
    { hide: u = [] } = e,
    { show: f = [1, 2, 3, -1, -2, -3] } = e,
    { nohi: h = 1 } = e;
  var d = 0,
    p,
    v,
    _,
    b = function (q, T, N, M) {
      if (h) return 0;
      var k = q.pflap + q.primer,
        E = T.pflap + T.primer,
        P;
      (Te.revcomp(E) + k, n == "delete" || n == "insert" || (n == "replace" && q.pflap.length > 0) ? (P = E.length) : (P = E.length + M));
      var G = (P % 3) + 1;
      return G;
    },
    w = function (q, T, N, M) {
      var k = "",
        E = "",
        P,
        G = [],
        O,
        K,
        D = [".", " .", "  .", "....", ".....", "......"],
        L = ["", " ", "  ", "   ", "    ", "     ", "      "],
        R;
      if (
        ((k = q.pflap.toUpperCase() + q.primer.toUpperCase()),
        (E = T.pflap.toUpperCase() + T.primer.toUpperCase()),
        (k = Te.revcomp(E) + k),
        t(4, (d = b(q, T, N, i))),
        k !== "")
      ) {
        for (P = k, K = Te.revcomp(P), O = 3; O >= 1; --O)
          if (f.length > 0 && f.indexOf(O) != -1 && P.length >= 3)
            try {
              G.push(
                D[O - 1] +
                  Te.translate(P, O - 1)
                    .split("")
                    .join(".."),
              );
            } catch {
              G.push("");
            }
          else G.push("");
        for (G[3] = P, K.length % 3, G[4] = K.split("").reverse().join(""), O = 1; O < 4; ++O)
          if (f.length > 0 && f.indexOf(-O) != -1 && K.length >= 3)
            try {
              ((R =
                D[O - 1] +
                Te.translate(K, O - 1)
                  .split("")
                  .join("..")),
                R.length < K.length && (R = R + L[K.length - R.length]),
                (R = R.split("").reverse().join("")),
                G.push(R));
            } catch {
              G.push("");
            }
          else G.push("");
      }
      return G;
    },
    y = function (q) {
      return { primer: q.seqfanneal, pflap: q.seqfpflap };
    },
    A = function (q) {
      return { primer: q.seqranneal, pflap: q.seqrpflap };
    };
  return (
    (l.$$set = (q) => {
      ("primerpair" in q && t(0, (s = q.primerpair)),
        "seqop" in q && t(1, (n = q.seqop)),
        "repseq" in q && t(2, (a = q.repseq)),
        "pad" in q && t(3, (i = q.pad)),
        "hide" in q && t(8, (u = q.hide)),
        "show" in q && t(9, (f = q.show)),
        "nohi" in q && t(10, (h = q.nohi)));
    }),
    (l.$$.update = () => {
      (l.$$.dirty & 16 && t(4, d),
        l.$$.dirty & 1 && t(5, (p = y(s))),
        l.$$.dirty & 1 && t(6, (v = A(s))),
        l.$$.dirty & 102 && t(7, (_ = w(p, v, a))));
    }),
    [s, n, a, i, d, p, v, _, u, f, h]
  );
}
class Ca extends Dl {
  constructor(e) {
    (super(), Il(this, e, Aa, ya, Ll, { primerpair: 0, seqop: 1, repseq: 2, pad: 3, hide: 8, show: 9, nohi: 10 }));
  }
}
var Ln = typeof globalThis < "u" ? globalThis : typeof window < "u" ? window : typeof global < "u" ? global : typeof self < "u" ? self : {},
  yo = { exports: {} };
(function (l, e) {
  (function (t, s) {
    s();
  })(Ln, function () {
    function t(h, d) {
      return (
        typeof d > "u" ? (d = { autoBom: !1 }) : typeof d != "object" && (d = { autoBom: !d }),
        d.autoBom && /^\s*(?:text\/\S*|application\/xml|\S*\/\S*\+xml)\s*;.*charset\s*=\s*utf-8/i.test(h.type)
          ? new Blob(["\uFEFF", h], { type: h.type })
          : h
      );
    }
    function s(h, d, p) {
      var v = new XMLHttpRequest();
      (v.open("GET", h),
        (v.responseType = "blob"),
        (v.onload = function () {
          f(v.response, d, p);
        }),
        (v.onerror = function () {}),
        v.send());
    }
    function n(h) {
      var d = new XMLHttpRequest();
      d.open("HEAD", h, !1);
      try {
        d.send();
      } catch {}
      return 200 <= d.status && 299 >= d.status;
    }
    function a(h) {
      try {
        h.dispatchEvent(new MouseEvent("click"));
      } catch {
        var d = document.createEvent("MouseEvents");
        (d.initMouseEvent("click", !0, !0, window, 0, 0, 0, 80, 20, !1, !1, !1, !1, 0, null), h.dispatchEvent(d));
      }
    }
    var i =
        typeof window == "object" && window.window === window
          ? window
          : typeof self == "object" && self.self === self
            ? self
            : typeof Ln == "object" && Ln.global === Ln
              ? Ln
              : void 0,
      u =
        i.navigator &&
        /Macintosh/.test(navigator.userAgent) &&
        /AppleWebKit/.test(navigator.userAgent) &&
        !/Safari/.test(navigator.userAgent),
      f =
        i.saveAs ||
        (typeof window != "object" || window !== i
          ? function () {}
          : "download" in HTMLAnchorElement.prototype && !u
            ? function (h, d, p) {
                var v = i.URL || i.webkitURL,
                  _ = document.createElement("a");
                ((d = d || h.name || "download"),
                  (_.download = d),
                  (_.rel = "noopener"),
                  typeof h == "string"
                    ? ((_.href = h), _.origin === location.origin ? a(_) : n(_.href) ? s(h, d, p) : a(_, (_.target = "_blank")))
                    : ((_.href = v.createObjectURL(h)),
                      setTimeout(function () {
                        v.revokeObjectURL(_.href);
                      }, 4e4),
                      setTimeout(function () {
                        a(_);
                      }, 0)));
              }
            : "msSaveOrOpenBlob" in navigator
              ? function (h, d, p) {
                  if (((d = d || h.name || "download"), typeof h != "string")) navigator.msSaveOrOpenBlob(t(h, p), d);
                  else if (n(h)) s(h, d, p);
                  else {
                    var v = document.createElement("a");
                    ((v.href = h),
                      (v.target = "_blank"),
                      setTimeout(function () {
                        a(v);
                      }));
                  }
                }
              : function (h, d, p, v) {
                  if (
                    ((v = v || open("", "_blank")),
                    v && (v.document.title = v.document.body.innerText = "downloading..."),
                    typeof h == "string")
                  )
                    return s(h, d, p);
                  var _ = h.type === "application/octet-stream",
                    b = /constructor/i.test(i.HTMLElement) || i.safari,
                    w = /CriOS\/[\d]+/.test(navigator.userAgent);
                  if ((w || (_ && b) || u) && typeof FileReader < "u") {
                    var y = new FileReader();
                    ((y.onloadend = function () {
                      var T = y.result;
                      ((T = w ? T : T.replace(/^data:[^;]*;/, "data:attachment/file;")),
                        v ? (v.location.href = T) : (location = T),
                        (v = null));
                    }),
                      y.readAsDataURL(h));
                  } else {
                    var A = i.URL || i.webkitURL,
                      q = A.createObjectURL(h);
                    (v ? (v.location = q) : (location.href = q),
                      (v = null),
                      setTimeout(function () {
                        A.revokeObjectURL(q);
                      }, 4e4));
                  }
                });
    ((i.saveAs = f.saveAs = f), (l.exports = f));
  });
})(yo);
const Ta = yo.exports;
function xi(l) {
  let e,
    t,
    s,
    n,
    a = l[1][0] + "",
    i,
    u,
    f,
    h,
    d,
    p = l[1][2] + "",
    v,
    _,
    b,
    w,
    y,
    A = l[0][2] + "",
    q,
    T,
    N,
    M,
    k,
    E,
    P,
    G,
    O,
    K,
    D = l[0][3].toUpperCase() + "",
    L,
    R,
    Q,
    ae,
    W = l[0][8] + "",
    x,
    H,
    U,
    le = l[1][1] + "",
    J,
    ie;
  return {
    c() {
      ((e = c("div")),
        (t = c("pre")),
        (s = C("")),
        (n = c("span")),
        (i = C(a)),
        (u = C(`
`)),
        (f = c("div")),
        (h = C(`
`)),
        (d = c("span")),
        (v = C(p)),
        (_ = C(`
`)),
        (b = C(`
`)),
        (w = c("div")),
        (y = C(`
`)),
        (q = C(A)),
        (T = C(`
`)),
        (N = C(`
`)),
        (M = S()),
        (k = c("div")),
        (E = c("pre")),
        (P = C("")),
        (G = c("div")),
        (O = C(`
`)),
        (K = c("span")),
        (L = C(D)),
        (R = C(`
`)),
        (Q = C(`
`)),
        (ae = c("span")),
        (x = C(W)),
        (H = C(`
`)),
        (U = c("span")),
        (J = C(le)),
        (ie = C(`
 
`)),
        r(d, "class", "cdn svelte-bfs72x"),
        r(f, "id", "codon_orf_fwd"),
        r(f, "class", "svelte-bfs72x"),
        r(w, "id", "codon_aa_fwd"),
        r(w, "class", "svelte-bfs72x"),
        r(t, "class", "svelte-bfs72x"),
        r(K, "class", "primer oligo  svelte-bfs72x"),
        r(G, "id", "codon_fwd_seq"),
        r(G, "class", "svelte-bfs72x"),
        r(ae, "class", "primer oligo f0b svelte-bfs72x"),
        r(E, "class", "svelte-bfs72x"),
        r(k, "class", "dsseqboxc svelte-bfs72x"),
        r(e, "id", "codontranslation"),
        r(e, "tabindex", "0"),
        r(e, "role", "region"),
        r(e, "aria-label", "Codon view: DNA and translation"),
        r(e, "class", "svelte-bfs72x"));
    },
    m(z, te) {
      (g(z, e, te),
        o(e, t),
        o(t, s),
        o(t, n),
        o(n, i),
        o(t, u),
        o(t, f),
        o(f, h),
        o(f, d),
        o(d, v),
        o(f, _),
        o(t, b),
        o(t, w),
        o(w, y),
        o(w, q),
        o(w, T),
        o(t, N),
        o(e, M),
        o(e, k),
        o(k, E),
        o(E, P),
        o(E, G),
        o(G, O),
        o(G, K),
        o(K, L),
        o(G, R),
        o(E, Q),
        o(E, ae),
        o(ae, x),
        o(E, H),
        o(E, U),
        o(U, J),
        o(E, ie));
    },
    p(z, te) {
      (te & 2 && a !== (a = z[1][0] + "") && Z(i, a),
        te & 2 && p !== (p = z[1][2] + "") && Z(v, p),
        te & 1 && A !== (A = z[0][2] + "") && Z(q, A),
        te & 1 && D !== (D = z[0][3].toUpperCase() + "") && Z(L, D),
        te & 1 && W !== (W = z[0][8] + "") && Z(x, W),
        te & 2 && le !== (le = z[1][1] + "") && Z(J, le));
    },
    d(z) {
      z && m(e);
    },
  };
}
function Sa(l) {
  let e,
    t = l[0].length >= 8 && xi(l);
  return {
    c() {
      (t && t.c(), (e = is()));
    },
    m(s, n) {
      (t && t.m(s, n), g(s, e, n));
    },
    p(s, [n]) {
      s[0].length >= 8 ? (t ? t.p(s, n) : ((t = xi(s)), t.c(), t.m(e.parentNode, e))) : t && (t.d(1), (t = null));
    },
    i: qe,
    o: qe,
    d(s) {
      (t && t.d(s), s && m(e));
    },
  };
}
function ka(l, e, t) {
  let { seq: s } = e,
    { seqrec: n } = e,
    { start: a = 1 } = e,
    { orfoffset: i = 1 } = e;
  var u,
    f = [];
  const h = function (_) {
      if (!_) return;
      var b = s.length,
        w = document.getElementById("codontranslation");
      if (!w) return;
      var y = w.offsetWidth,
        A = w.scrollLeft;
      const q = d();
      let T = Math.ceil(A / q),
        N = Math.ceil(y / q) + T;
      var M = _ * q;
      (_ < T || _ > N - 2 * q) && (w.scrollLeft = M);
    },
    d = function () {
      const _ = document.body,
        b = document.createElement("span"),
        w = 3;
      ((b.style.width = "1ch"),
        (b.style.position = "fixed"),
        (b.style.fontFamily = "monospace"),
        (b.style.fontSize = "16px"),
        _.appendChild(b));
      const y = b.getBoundingClientRect().width;
      return (_.removeChild(b), y + w);
    };
  var p = function (_) {
      _ = _ || "";
      var b = "",
        w = "",
        y = [],
        A,
        q,
        T = [".", " .", "  .", "....", ".....", "......"],
        N = ["", " ", "  ", "   ", "    ", "     ", "      "],
        M;
      ((b = _), (w = Te.revcomp(b)));
      try {
        if (b !== "") {
          for (A = 2; A >= 0; --A) y.push(T[A] + Te.translate(b, A).split("").join(".."));
          for (y[3] = b, q = w.length % 3, y[4] = w.split("").reverse().join(""), A = 0; A < 3; ++A)
            ((M = T[A] + Te.translate(w, A).split("").join("..")),
              M.length < w.length && (M = M + N[w.length - M.length]),
              (M = M.split("").reverse().join("")),
              y.push(M));
        }
        ((y[8] = b.slice().replace(/\w/g, "\u25B2")), (y[9] = w.slice().replace(/\w/g, "\u25BC")));
      } catch {
        y = T;
      }
      return y;
    },
    v = function (_) {
      t(2, (a = a || 0));
      var b = "",
        w = "",
        y = "",
        A = "<\u25FB\uFE0E>";
      A = "<=>";
      var q;
      if (_.length < 8) return "";
      var T = _[3].length,
        N = (_[2].indexOf("*") != -1 ? _[2].indexOf("*") - 1 : T) / 3;
      N < 0 && (N = 0);
      for (let M = 0; M < N; M += 5) M > 0 ? (b += " |" + String(M).padEnd(13, " ")) : (b += " |" + String(1).padEnd(10, " "));
      y = A.repeat(N);
      for (let M = 0; M < T; M += 10)
        M > 0
          ? ((q = (M + a) % n.seq.length), (q = q == 0 ? n.seq.length : q), (w += "|" + String(q).padEnd(9)))
          : ((q = (1 + a) % n.seq.length), (q = q == 0 ? n.seq.length : q), (w += "|" + String(q).padEnd(8)));
      return [b, w, y];
    };
  return (
    (l.$$set = (_) => {
      ("seq" in _ && t(3, (s = _.seq)),
        "seqrec" in _ && t(4, (n = _.seqrec)),
        "start" in _ && t(2, (a = _.start)),
        "orfoffset" in _ && t(5, (i = _.orfoffset)));
    }),
    (l.$$.update = () => {
      (l.$$.dirty & 8 && t(0, (u = p(s))), l.$$.dirty & 1 && t(1, (f = v(u))));
    }),
    [u, f, a, s, n, i, h]
  );
}
class Ao extends Dl {
  constructor(e) {
    (super(), Il(this, e, ka, Sa, Ll, { seq: 3, seqrec: 4, start: 2, orfoffset: 5, setScrollPos: 6 }));
  }
  get setScrollPos() {
    return this.$$.ctx[6];
  }
}
function Ji(l) {
  let e,
    t,
    s,
    n = l[2][0] + "",
    a,
    i,
    u = l[2][1] + "",
    f,
    h,
    d = l[2][2] + "",
    p,
    v,
    _,
    b,
    w,
    y,
    A,
    q,
    T,
    N,
    M,
    k,
    E = l[2][5] + "",
    P,
    G,
    O = l[2][6] + "",
    K,
    D,
    L = l[2][7] + "",
    R,
    Q,
    ae,
    W;
  function x(le, J) {
    return (
      J & 3 && (A = null),
      J & 3 && (q = null),
      A == null && (A = !isNaN(le[0]) && !isNaN(le[1]) && le[0] <= le[1]),
      A ? Na : (q == null && (q = !isNaN(le[0]) && !isNaN(le[1]) && le[0] > le[1]), q ? Ea : Ma)
    );
  }
  let H = x(l, -1),
    U = H(l);
  return {
    c() {
      ((e = c("div")),
        (t = c("pre")),
        (s = C("")),
        (a = C(n)),
        (i = C(`
`)),
        (f = C(u)),
        (h = C(`
`)),
        (p = C(d)),
        (v = C(`
`)),
        (_ = S()),
        (b = c("div")),
        (w = c("pre")),
        (y = C("")),
        U.c(),
        (T = C(`
`)),
        (N = S()),
        (M = c("pre")),
        (k = C("")),
        (P = C(E)),
        (G = C(`
`)),
        (K = C(O)),
        (D = C(`
`)),
        (R = C(L)),
        (Q = C(`
`)),
        (ae = S()),
        (W = c("br")),
        r(t, "class", "svelte-16xlwqz"),
        r(w, "class", "svelte-16xlwqz"),
        r(b, "id", "dsseqbox"),
        r(b, "class", "dsseqbox svelte-16xlwqz"),
        r(M, "class", "svelte-16xlwqz"),
        r(e, "id", "indeltranslation"),
        r(e, "tabindex", "0"),
        r(e, "role", "region"),
        r(e, "aria-label", "Sequence view"),
        r(e, "class", "svelte-16xlwqz"));
    },
    m(le, J) {
      (g(le, e, J),
        o(e, t),
        o(t, s),
        o(t, a),
        o(t, i),
        o(t, f),
        o(t, h),
        o(t, p),
        o(t, v),
        o(e, _),
        o(e, b),
        o(b, w),
        o(w, y),
        U.m(w, null),
        o(w, T),
        o(e, N),
        o(e, M),
        o(M, k),
        o(M, P),
        o(M, G),
        o(M, K),
        o(M, D),
        o(M, R),
        o(M, Q),
        o(e, ae),
        o(e, W));
    },
    p(le, J) {
      (J & 4 && n !== (n = le[2][0] + "") && Z(a, n),
        J & 4 && u !== (u = le[2][1] + "") && Z(f, u),
        J & 4 && d !== (d = le[2][2] + "") && Z(p, d),
        H === (H = x(le, J)) && U ? U.p(le, J) : (U.d(1), (U = H(le)), U && (U.c(), U.m(w, T))),
        J & 4 && E !== (E = le[2][5] + "") && Z(P, E),
        J & 4 && O !== (O = le[2][6] + "") && Z(K, O),
        J & 4 && L !== (L = le[2][7] + "") && Z(R, L));
    },
    d(le) {
      (le && m(e), U.d());
    },
  };
}
function Ma(l) {
  let e,
    t,
    s,
    n,
    a = l[2][3] + "",
    i,
    u,
    f,
    h,
    d = l[2][8] + "",
    p,
    v,
    _,
    b = l[3][0] + "",
    w,
    y,
    A,
    q = l[2][9] + "",
    T,
    N,
    M,
    k,
    E,
    P,
    G = l[2][4] + "",
    O,
    K;
  return {
    c() {
      ((e = c("div")),
        (t = C(`
`)),
        (s = c("span")),
        (n = c("b")),
        (i = C(a)),
        (u = C(`
`)),
        (f = C(`
`)),
        (h = c("span")),
        (p = C(d)),
        (v = C(`
`)),
        (_ = c("span")),
        (w = C(b)),
        (y = C(`
`)),
        (A = c("span")),
        (T = C(q)),
        (N = C(`
`)),
        (M = c("div")),
        (k = C(`
`)),
        (E = c("span")),
        (P = c("b")),
        (O = C(G)),
        (K = C(`
`)),
        r(s, "class", "primer oligo svelte-16xlwqz"),
        r(e, "id", "indel_fwd_seq"),
        r(e, "class", "svelte-16xlwqz"),
        r(h, "class", "primer oligo f0b svelte-16xlwqz"),
        r(A, "class", "primer oligo f0b svelte-16xlwqz"),
        r(E, "class", "primer oligo svelte-16xlwqz"),
        r(M, "id", "indel_rev_seq"),
        r(M, "class", "svelte-16xlwqz"));
    },
    m(D, L) {
      (g(D, e, L),
        o(e, t),
        o(e, s),
        o(s, n),
        o(n, i),
        o(e, u),
        g(D, f, L),
        g(D, h, L),
        o(h, p),
        g(D, v, L),
        g(D, _, L),
        o(_, w),
        g(D, y, L),
        g(D, A, L),
        o(A, T),
        g(D, N, L),
        g(D, M, L),
        o(M, k),
        o(M, E),
        o(E, P),
        o(P, O),
        o(M, K));
    },
    p(D, L) {
      (L & 4 && a !== (a = D[2][3] + "") && Z(i, a),
        L & 4 && d !== (d = D[2][8] + "") && Z(p, d),
        L & 8 && b !== (b = D[3][0] + "") && Z(w, b),
        L & 4 && q !== (q = D[2][9] + "") && Z(T, q),
        L & 4 && G !== (G = D[2][4] + "") && Z(O, G));
    },
    d(D) {
      (D && m(e), D && m(f), D && m(h), D && m(v), D && m(_), D && m(y), D && m(A), D && m(N), D && m(M));
    },
  };
}
function Ea(l) {
  let e,
    t,
    s,
    n,
    a = l[2][3] + "",
    i,
    u,
    f,
    h,
    d = l[2][8].slice(0, l[1]) + "",
    p,
    v,
    _ = l[2][8][l[1]] + "",
    b,
    w,
    y = l[2][8].slice(l[1] + 1, l[0]) + "",
    A,
    q,
    T = l[2][8][l[0]] + "",
    N,
    M,
    k = l[2][8].slice(l[0] + 1) + "",
    E,
    P,
    G,
    O = l[3][0] + "",
    K,
    D,
    L,
    R = l[2][9].slice(0, l[1]) + "",
    Q,
    ae,
    W = l[2][9][l[1]] + "",
    x,
    H,
    U = l[2][9].slice(l[1] + 1, l[0]) + "",
    le,
    J,
    ie = l[2][9][l[0]] + "",
    z,
    te,
    pe = l[2][9].slice(l[0] + 1) + "",
    Y,
    ue,
    he,
    de,
    _e,
    ne,
    $ = l[2][4] + "",
    fe,
    we;
  return {
    c() {
      ((e = c("div")),
        (t = C(`
`)),
        (s = c("span")),
        (n = c("b")),
        (i = C(a)),
        (u = C(`
`)),
        (f = C(`
`)),
        (h = c("span")),
        (p = C(d)),
        (v = c("span")),
        (b = C(_)),
        (w = c("span")),
        (A = C(y)),
        (q = c("span")),
        (N = C(T)),
        (M = c("span")),
        (E = C(k)),
        (P = C(`
`)),
        (G = c("span")),
        (K = C(O)),
        (D = C(`
`)),
        (L = c("span")),
        (Q = C(R)),
        (ae = c("span")),
        (x = C(W)),
        (H = c("span")),
        (le = C(U)),
        (J = c("span")),
        (z = C(ie)),
        (te = c("span")),
        (Y = C(pe)),
        (ue = C(`
`)),
        (he = c("div")),
        (de = C(`
`)),
        (_e = c("span")),
        (ne = c("b")),
        (fe = C($)),
        (we = C(`
`)),
        r(s, "class", "primer oligo svelte-16xlwqz"),
        r(e, "id", "indel_fwd_seq"),
        r(e, "class", "svelte-16xlwqz"),
        r(h, "class", "primer oligo f4b svelte-16xlwqz"),
        r(v, "class", "primer oligo f1b svelte-16xlwqz"),
        r(w, "class", "primer oligo f0b svelte-16xlwqz"),
        r(q, "class", "primer oligo f1b svelte-16xlwqz"),
        r(M, "class", "primer oligo f4b svelte-16xlwqz"),
        r(L, "class", "primer oligo f4b svelte-16xlwqz"),
        r(ae, "class", "primer oligo f1b svelte-16xlwqz"),
        r(H, "class", "primer oligo f0b svelte-16xlwqz"),
        r(J, "class", "primer oligo f1b svelte-16xlwqz"),
        r(te, "class", "primer oligo f4b svelte-16xlwqz"),
        r(_e, "class", "primer oligo svelte-16xlwqz"),
        r(he, "id", "indel_rev_seq"),
        r(he, "class", "svelte-16xlwqz"));
    },
    m(j, F) {
      (g(j, e, F),
        o(e, t),
        o(e, s),
        o(s, n),
        o(n, i),
        o(e, u),
        g(j, f, F),
        g(j, h, F),
        o(h, p),
        g(j, v, F),
        o(v, b),
        g(j, w, F),
        o(w, A),
        g(j, q, F),
        o(q, N),
        g(j, M, F),
        o(M, E),
        g(j, P, F),
        g(j, G, F),
        o(G, K),
        g(j, D, F),
        g(j, L, F),
        o(L, Q),
        g(j, ae, F),
        o(ae, x),
        g(j, H, F),
        o(H, le),
        g(j, J, F),
        o(J, z),
        g(j, te, F),
        o(te, Y),
        g(j, ue, F),
        g(j, he, F),
        o(he, de),
        o(he, _e),
        o(_e, ne),
        o(ne, fe),
        o(he, we));
    },
    p(j, F) {
      (F & 4 && a !== (a = j[2][3] + "") && Z(i, a),
        F & 6 && d !== (d = j[2][8].slice(0, j[1]) + "") && Z(p, d),
        F & 6 && _ !== (_ = j[2][8][j[1]] + "") && Z(b, _),
        F & 7 && y !== (y = j[2][8].slice(j[1] + 1, j[0]) + "") && Z(A, y),
        F & 5 && T !== (T = j[2][8][j[0]] + "") && Z(N, T),
        F & 5 && k !== (k = j[2][8].slice(j[0] + 1) + "") && Z(E, k),
        F & 8 && O !== (O = j[3][0] + "") && Z(K, O),
        F & 6 && R !== (R = j[2][9].slice(0, j[1]) + "") && Z(Q, R),
        F & 6 && W !== (W = j[2][9][j[1]] + "") && Z(x, W),
        F & 7 && U !== (U = j[2][9].slice(j[1] + 1, j[0]) + "") && Z(le, U),
        F & 5 && ie !== (ie = j[2][9][j[0]] + "") && Z(z, ie),
        F & 5 && pe !== (pe = j[2][9].slice(j[0] + 1) + "") && Z(Y, pe),
        F & 4 && $ !== ($ = j[2][4] + "") && Z(fe, $));
    },
    d(j) {
      (j && m(e),
        j && m(f),
        j && m(h),
        j && m(v),
        j && m(w),
        j && m(q),
        j && m(M),
        j && m(P),
        j && m(G),
        j && m(D),
        j && m(L),
        j && m(ae),
        j && m(H),
        j && m(J),
        j && m(te),
        j && m(ue),
        j && m(he));
    },
  };
}
function Na(l) {
  let e,
    t,
    s,
    n,
    a = l[2][3] + "",
    i,
    u,
    f,
    h,
    d = l[2][8].slice(0, l[0]) + "",
    p,
    v,
    _ = l[2][8][l[0]] + "",
    b,
    w,
    y = l[2][8].slice(l[0] + 1, l[1]) + "",
    A,
    q,
    T = l[2][8][l[1]] + "",
    N,
    M,
    k = l[2][8].slice(l[1] + 1) + "",
    E,
    P,
    G,
    O = l[3][0] + "",
    K,
    D,
    L,
    R = l[2][9].slice(0, l[0]) + "",
    Q,
    ae,
    W = l[2][9][l[0]] + "",
    x,
    H,
    U = l[2][9].slice(l[0] + 1, l[1]) + "",
    le,
    J,
    ie = l[2][9][l[1]] + "",
    z,
    te,
    pe = l[2][9].slice(l[1] + 1) + "",
    Y,
    ue,
    he,
    de,
    _e,
    ne,
    $ = l[2][4] + "",
    fe,
    we;
  return {
    c() {
      ((e = c("div")),
        (t = C(`
`)),
        (s = c("span")),
        (n = c("b")),
        (i = C(a)),
        (u = C(`
`)),
        (f = C(`
`)),
        (h = c("span")),
        (p = C(d)),
        (v = c("span")),
        (b = C(_)),
        (w = c("span")),
        (A = C(y)),
        (q = c("span")),
        (N = C(T)),
        (M = c("span")),
        (E = C(k)),
        (P = C(`
`)),
        (G = c("span")),
        (K = C(O)),
        (D = C(`
`)),
        (L = c("span")),
        (Q = C(R)),
        (ae = c("span")),
        (x = C(W)),
        (H = c("span")),
        (le = C(U)),
        (J = c("span")),
        (z = C(ie)),
        (te = c("span")),
        (Y = C(pe)),
        (ue = C(`
`)),
        (he = c("div")),
        (de = C(`
`)),
        (_e = c("span")),
        (ne = c("b")),
        (fe = C($)),
        (we = C(`
`)),
        r(s, "class", "primer oligo svelte-16xlwqz"),
        r(e, "id", "indel_fwd_seq"),
        r(e, "class", "svelte-16xlwqz"),
        r(h, "class", "primer oligo f0b svelte-16xlwqz"),
        r(v, "class", "primer oligo f1b svelte-16xlwqz"),
        r(w, "class", "primer oligo f4b svelte-16xlwqz"),
        r(q, "class", "primer oligo f1b svelte-16xlwqz"),
        r(M, "class", "primer oligo f0b svelte-16xlwqz"),
        r(L, "class", "primer oligo f0b svelte-16xlwqz"),
        r(ae, "class", "primer oligo f1b svelte-16xlwqz"),
        r(H, "class", "primer oligo f4b svelte-16xlwqz"),
        r(J, "class", "primer oligo f1b svelte-16xlwqz"),
        r(te, "class", "primer oligo f0b svelte-16xlwqz"),
        r(_e, "class", "primer oligo svelte-16xlwqz"),
        r(he, "id", "indel_rev_seq"),
        r(he, "class", "svelte-16xlwqz"));
    },
    m(j, F) {
      (g(j, e, F),
        o(e, t),
        o(e, s),
        o(s, n),
        o(n, i),
        o(e, u),
        g(j, f, F),
        g(j, h, F),
        o(h, p),
        g(j, v, F),
        o(v, b),
        g(j, w, F),
        o(w, A),
        g(j, q, F),
        o(q, N),
        g(j, M, F),
        o(M, E),
        g(j, P, F),
        g(j, G, F),
        o(G, K),
        g(j, D, F),
        g(j, L, F),
        o(L, Q),
        g(j, ae, F),
        o(ae, x),
        g(j, H, F),
        o(H, le),
        g(j, J, F),
        o(J, z),
        g(j, te, F),
        o(te, Y),
        g(j, ue, F),
        g(j, he, F),
        o(he, de),
        o(he, _e),
        o(_e, ne),
        o(ne, fe),
        o(he, we));
    },
    p(j, F) {
      (F & 4 && a !== (a = j[2][3] + "") && Z(i, a),
        F & 5 && d !== (d = j[2][8].slice(0, j[0]) + "") && Z(p, d),
        F & 5 && _ !== (_ = j[2][8][j[0]] + "") && Z(b, _),
        F & 7 && y !== (y = j[2][8].slice(j[0] + 1, j[1]) + "") && Z(A, y),
        F & 6 && T !== (T = j[2][8][j[1]] + "") && Z(N, T),
        F & 6 && k !== (k = j[2][8].slice(j[1] + 1) + "") && Z(E, k),
        F & 8 && O !== (O = j[3][0] + "") && Z(K, O),
        F & 5 && R !== (R = j[2][9].slice(0, j[0]) + "") && Z(Q, R),
        F & 5 && W !== (W = j[2][9][j[0]] + "") && Z(x, W),
        F & 7 && U !== (U = j[2][9].slice(j[0] + 1, j[1]) + "") && Z(le, U),
        F & 6 && ie !== (ie = j[2][9][j[1]] + "") && Z(z, ie),
        F & 6 && pe !== (pe = j[2][9].slice(j[1] + 1) + "") && Z(Y, pe),
        F & 4 && $ !== ($ = j[2][4] + "") && Z(fe, $));
    },
    d(j) {
      (j && m(e),
        j && m(f),
        j && m(h),
        j && m(v),
        j && m(w),
        j && m(q),
        j && m(M),
        j && m(P),
        j && m(G),
        j && m(D),
        j && m(L),
        j && m(ae),
        j && m(H),
        j && m(J),
        j && m(te),
        j && m(ue),
        j && m(he));
    },
  };
}
function Pa(l) {
  let e,
    t = l[2].length >= 8 && Ji(l);
  return {
    c() {
      (t && t.c(), (e = is()));
    },
    m(s, n) {
      (t && t.m(s, n), g(s, e, n));
    },
    p(s, [n]) {
      s[2].length >= 8 ? (t ? t.p(s, n) : ((t = Ji(s)), t.c(), t.m(e.parentNode, e))) : t && (t.d(1), (t = null));
    },
    i: qe,
    o: qe,
    d(s) {
      (t && t.d(s), s && m(e));
    },
  };
}
function La(l, e, t) {
  let { seq: s } = e,
    { start: n } = e,
    { end: a } = e;
  const i = function (v) {
      if (!v) return;
      var _ = s.length,
        b = document.getElementById("indeltranslation");
      if (!b) return;
      var w = b.offsetWidth,
        y = b.scrollLeft;
      const A = u();
      let q = Math.ceil(y / A),
        T = Math.ceil(w / A) + q;
      var N = v * A;
      (v < q || v > T) && (b.scrollLeft = N);
    },
    u = function () {
      const v = document.body,
        _ = document.createElement("span"),
        b = 3;
      ((_.style.width = "1ch"),
        (_.style.position = "fixed"),
        (_.style.fontFamily = "monospace"),
        (_.style.fontSize = "16px"),
        v.appendChild(_));
      const w = _.getBoundingClientRect().width;
      return (v.removeChild(_), w + b);
    };
  var f,
    h = [],
    d = function (v) {
      v = v || "";
      var _ = "",
        b = "",
        w = [],
        y,
        A = [".", " .", "  .", "....", ".....", "......"],
        q = ["", " ", "  ", "   ", "    ", "     ", "      "],
        T;
      if (((_ = v), (b = Te.revcomp(_)), _ !== "")) {
        for (y = 2; y >= 0; --y) w.push(A[y] + Te.translate(_, y).split("").join(".."));
        for (w[3] = _, b.length % 3, w[4] = b.split("").reverse().join(""), y = 0; y < 3; ++y)
          ((T = A[y] + Te.translate(b, y).split("").join("..")),
            T.length < b.length && (T = T + q[b.length - T.length]),
            (T = T.split("").reverse().join("")),
            w.push(T));
      }
      return ((w[8] = _.slice().replace(/\w/g, "\u25B2")), (w[9] = b.slice().replace(/\w/g, "\u25BC")), w);
    },
    p = function (v) {
      var _ = "";
      if (v.length < 8) return "";
      var b = v[3].length;
      for (let w = 0; w < b; w += 10) w > 0 ? (_ += "|" + String(w).padEnd(9)) : (_ += "|" + String(1).padEnd(8));
      return [_];
    };
  return (
    document.getElementById("slide"),
    (l.$$set = (v) => {
      ("seq" in v && t(4, (s = v.seq)), "start" in v && t(0, (n = v.start)), "end" in v && t(1, (a = v.end)));
    }),
    (l.$$.update = () => {
      (l.$$.dirty & 16 && t(2, (f = d(s))),
        l.$$.dirty & 4 && t(3, (h = p(f))),
        l.$$.dirty & 1 && t(0, (n = Number(n))),
        l.$$.dirty & 1 && i(n),
        l.$$.dirty & 2 && t(1, (a = Number(a))));
    }),
    [n, a, f, h, s, i]
  );
}
class Ga extends Dl {
  constructor(e) {
    (super(), Il(this, e, La, Pa, Ll, { seq: 4, start: 0, end: 1, setScrollPos: 5 }));
  }
  get setScrollPos() {
    return this.$$.ctx[5];
  }
}
function Zi(l) {
  let e,
    t,
    s,
    n = l[2][0] + "",
    a,
    i,
    u = l[2][1] + "",
    f,
    h,
    d = l[2][2] + "",
    p,
    v,
    _,
    b,
    w,
    y,
    A,
    q,
    T,
    N,
    M,
    k,
    E;
  function P(K, D) {
    return (
      D & 3 && (A = null),
      D & 3 && (q = null),
      A == null && (A = !isNaN(K[0]) && !isNaN(K[1]) && K[0] <= K[1]),
      A ? Ia : (q == null && (q = !isNaN(K[0]) && !isNaN(K[1]) && K[0] > K[1]), q ? Ra : Fa)
    );
  }
  let G = P(l, -1),
    O = G(l);
  return {
    c() {
      ((e = c("div")),
        (t = c("pre")),
        (s = C("")),
        (a = C(n)),
        (i = C(`
`)),
        (f = C(u)),
        (h = C(`
`)),
        (p = C(d)),
        (v = C(`
`)),
        (_ = S()),
        (b = c("div")),
        (w = c("pre")),
        (y = C("")),
        O.c(),
        (T = C(`
`)),
        (N = S()),
        (M = c("pre")),
        (M.textContent = `
`),
        (k = S()),
        (E = c("br")),
        r(t, "class", "svelte-b3eop4"),
        r(w, "class", "svelte-b3eop4"),
        r(b, "id", "dsseqboxl"),
        r(b, "class", "dsseqboxl svelte-b3eop4"),
        r(M, "class", "svelte-b3eop4"),
        r(e, "id", "legacytranslation"),
        r(e, "tabindex", "0"),
        r(e, "role", "region"),
        r(e, "aria-label", "Sequence view"),
        r(e, "class", "svelte-b3eop4"));
    },
    m(K, D) {
      (g(K, e, D),
        o(e, t),
        o(t, s),
        o(t, a),
        o(t, i),
        o(t, f),
        o(t, h),
        o(t, p),
        o(t, v),
        o(e, _),
        o(e, b),
        o(b, w),
        o(w, y),
        O.m(w, null),
        o(w, T),
        o(e, N),
        o(e, M),
        o(e, k),
        o(e, E));
    },
    p(K, D) {
      (D & 4 && n !== (n = K[2][0] + "") && Z(a, n),
        D & 4 && u !== (u = K[2][1] + "") && Z(f, u),
        D & 4 && d !== (d = K[2][2] + "") && Z(p, d),
        G === (G = P(K, D)) && O ? O.p(K, D) : (O.d(1), (O = G(K)), O && (O.c(), O.m(w, T))));
    },
    d(K) {
      (K && m(e), O.d());
    },
  };
}
function Fa(l) {
  let e,
    t,
    s,
    n,
    a = l[2][3] + "",
    i,
    u,
    f,
    h,
    d = l[2][8] + "",
    p,
    v,
    _,
    b = l[3][0] + "",
    w,
    y;
  return {
    c() {
      ((e = c("div")),
        (t = C(`
`)),
        (s = c("span")),
        (n = c("b")),
        (i = C(a)),
        (u = C(`
`)),
        (f = C(`
`)),
        (h = c("span")),
        (p = C(d)),
        (v = C(`
`)),
        (_ = c("span")),
        (w = C(b)),
        (y = C(`
`)),
        r(s, "class", "primer oligo svelte-b3eop4"),
        r(e, "id", "legacy_fwd_seq"),
        r(e, "class", "svelte-b3eop4"),
        r(h, "class", "primer oligo f0b svelte-b3eop4"));
    },
    m(A, q) {
      (g(A, e, q),
        o(e, t),
        o(e, s),
        o(s, n),
        o(n, i),
        o(e, u),
        g(A, f, q),
        g(A, h, q),
        o(h, p),
        g(A, v, q),
        g(A, _, q),
        o(_, w),
        g(A, y, q));
    },
    p(A, q) {
      (q & 4 && a !== (a = A[2][3] + "") && Z(i, a),
        q & 4 && d !== (d = A[2][8] + "") && Z(p, d),
        q & 8 && b !== (b = A[3][0] + "") && Z(w, b));
    },
    d(A) {
      (A && m(e), A && m(f), A && m(h), A && m(v), A && m(_), A && m(y));
    },
  };
}
function Ra(l) {
  let e,
    t,
    s,
    n,
    a = l[2][3] + "",
    i,
    u,
    f,
    h,
    d = l[2][8].slice(0, l[1]) + "",
    p,
    v,
    _ = l[2][8][l[1]] + "",
    b,
    w,
    y = l[2][8].slice(l[1] + 1, l[0]) + "",
    A,
    q,
    T = l[2][8][l[0]] + "",
    N,
    M,
    k = l[2][8].slice(l[0] + 1) + "",
    E,
    P,
    G,
    O = l[3][0] + "",
    K,
    D;
  return {
    c() {
      ((e = c("div")),
        (t = C(`
`)),
        (s = c("span")),
        (n = c("b")),
        (i = C(a)),
        (u = C(`
`)),
        (f = C(`
`)),
        (h = c("span")),
        (p = C(d)),
        (v = c("span")),
        (b = C(_)),
        (w = c("span")),
        (A = C(y)),
        (q = c("span")),
        (N = C(T)),
        (M = c("span")),
        (E = C(k)),
        (P = C(`
`)),
        (G = c("span")),
        (K = C(O)),
        (D = C(`
`)),
        r(s, "class", "primer oligo svelte-b3eop4"),
        r(e, "id", "legacy_fwd_seq"),
        r(e, "class", "svelte-b3eop4"),
        r(h, "class", "primer oligo f4b svelte-b3eop4"),
        r(v, "class", "primer oligo f1b svelte-b3eop4"),
        r(w, "class", "primer oligo f0b svelte-b3eop4"),
        r(q, "class", "primer oligo f1b svelte-b3eop4"),
        r(M, "class", "primer oligo f4b svelte-b3eop4"));
    },
    m(L, R) {
      (g(L, e, R),
        o(e, t),
        o(e, s),
        o(s, n),
        o(n, i),
        o(e, u),
        g(L, f, R),
        g(L, h, R),
        o(h, p),
        g(L, v, R),
        o(v, b),
        g(L, w, R),
        o(w, A),
        g(L, q, R),
        o(q, N),
        g(L, M, R),
        o(M, E),
        g(L, P, R),
        g(L, G, R),
        o(G, K),
        g(L, D, R));
    },
    p(L, R) {
      (R & 4 && a !== (a = L[2][3] + "") && Z(i, a),
        R & 6 && d !== (d = L[2][8].slice(0, L[1]) + "") && Z(p, d),
        R & 6 && _ !== (_ = L[2][8][L[1]] + "") && Z(b, _),
        R & 7 && y !== (y = L[2][8].slice(L[1] + 1, L[0]) + "") && Z(A, y),
        R & 5 && T !== (T = L[2][8][L[0]] + "") && Z(N, T),
        R & 5 && k !== (k = L[2][8].slice(L[0] + 1) + "") && Z(E, k),
        R & 8 && O !== (O = L[3][0] + "") && Z(K, O));
    },
    d(L) {
      (L && m(e), L && m(f), L && m(h), L && m(v), L && m(w), L && m(q), L && m(M), L && m(P), L && m(G), L && m(D));
    },
  };
}
function Ia(l) {
  let e,
    t,
    s,
    n,
    a = l[2][3] + "",
    i,
    u,
    f,
    h,
    d = l[2][8].slice(0, l[0]) + "",
    p,
    v,
    _ = l[2][8][l[0]] + "",
    b,
    w,
    y = l[2][8].slice(l[0] + 1, l[1]) + "",
    A,
    q,
    T = l[2][8][l[1]] + "",
    N,
    M,
    k = l[2][8].slice(l[1] + 1) + "",
    E,
    P,
    G,
    O = l[3][0] + "",
    K,
    D;
  return {
    c() {
      ((e = c("div")),
        (t = C(`
`)),
        (s = c("span")),
        (n = c("b")),
        (i = C(a)),
        (u = C(`
`)),
        (f = C(`
`)),
        (h = c("span")),
        (p = C(d)),
        (v = c("span")),
        (b = C(_)),
        (w = c("span")),
        (A = C(y)),
        (q = c("span")),
        (N = C(T)),
        (M = c("span")),
        (E = C(k)),
        (P = C(`
`)),
        (G = c("span")),
        (K = C(O)),
        (D = C(`
`)),
        r(s, "class", "primer oligo svelte-b3eop4"),
        r(e, "id", "legacy_fwd_seq"),
        r(e, "class", "svelte-b3eop4"),
        r(h, "class", "primer oligo f0b svelte-b3eop4"),
        r(v, "class", "primer oligo f1b svelte-b3eop4"),
        r(w, "class", "primer oligo f4b svelte-b3eop4"),
        r(q, "class", "primer oligo f1b svelte-b3eop4"),
        r(M, "class", "primer oligo f0b svelte-b3eop4"));
    },
    m(L, R) {
      (g(L, e, R),
        o(e, t),
        o(e, s),
        o(s, n),
        o(n, i),
        o(e, u),
        g(L, f, R),
        g(L, h, R),
        o(h, p),
        g(L, v, R),
        o(v, b),
        g(L, w, R),
        o(w, A),
        g(L, q, R),
        o(q, N),
        g(L, M, R),
        o(M, E),
        g(L, P, R),
        g(L, G, R),
        o(G, K),
        g(L, D, R));
    },
    p(L, R) {
      (R & 4 && a !== (a = L[2][3] + "") && Z(i, a),
        R & 5 && d !== (d = L[2][8].slice(0, L[0]) + "") && Z(p, d),
        R & 5 && _ !== (_ = L[2][8][L[0]] + "") && Z(b, _),
        R & 7 && y !== (y = L[2][8].slice(L[0] + 1, L[1]) + "") && Z(A, y),
        R & 6 && T !== (T = L[2][8][L[1]] + "") && Z(N, T),
        R & 6 && k !== (k = L[2][8].slice(L[1] + 1) + "") && Z(E, k),
        R & 8 && O !== (O = L[3][0] + "") && Z(K, O));
    },
    d(L) {
      (L && m(e), L && m(f), L && m(h), L && m(v), L && m(w), L && m(q), L && m(M), L && m(P), L && m(G), L && m(D));
    },
  };
}
function Da(l) {
  let e,
    t = l[2].length >= 8 && Zi(l);
  return {
    c() {
      (t && t.c(), (e = is()));
    },
    m(s, n) {
      (t && t.m(s, n), g(s, e, n));
    },
    p(s, [n]) {
      s[2].length >= 8 ? (t ? t.p(s, n) : ((t = Zi(s)), t.c(), t.m(e.parentNode, e))) : t && (t.d(1), (t = null));
    },
    i: qe,
    o: qe,
    d(s) {
      (t && t.d(s), s && m(e));
    },
  };
}
function Oa(l, e, t) {
  let { seq: s } = e,
    { start: n } = e,
    { end: a } = e,
    { selstart: i } = e,
    { selend: u } = e;
  const f = function (b) {
      if (!b) return;
      var w = s.length,
        y = document.getElementById("legacytranslation");
      if (!y) return;
      var A = y.offsetWidth,
        q = y.scrollLeft;
      const T = h();
      let N = Math.ceil(q / T),
        M = Math.ceil(A / T) + N;
      var k = b * T;
      (b < N || b > M) && (y.scrollLeft = k);
    },
    h = function () {
      const b = document.body,
        w = document.createElement("span"),
        y = 3;
      ((w.style.width = "1ch"),
        (w.style.position = "fixed"),
        (w.style.fontFamily = "monospace"),
        (w.style.fontSize = "16px"),
        b.appendChild(w));
      const A = w.getBoundingClientRect().width;
      return (b.removeChild(w), A + y);
    };
  var d,
    p = [],
    v = function (b) {
      b = b || "";
      var w = "",
        y = "",
        A = [],
        q,
        T = [".", " .", "  .", "....", ".....", "......"],
        N = ["", " ", "  ", "   ", "    ", "     ", "      "],
        M;
      if (((w = b), (y = Te.revcomp(w)), w !== "")) {
        for (q = 2; q >= 0; --q) A.push(T[q] + Te.translate(w, q).split("").join(".."));
        for (A[3] = w, y.length % 3, A[4] = y.split("").reverse().join(""), q = 0; q < 3; ++q)
          ((M = T[q] + Te.translate(y, q).split("").join("..")),
            M.length < y.length && (M = M + N[y.length - M.length]),
            (M = M.split("").reverse().join("")),
            A.push(M));
      }
      return ((A[8] = w.slice().replace(/\w/g, "\u25B2")), (A[9] = y.slice().replace(/\w/g, "\u25BC")), A);
    },
    _ = function (b) {
      var w = "";
      if (b.length < 8) return "";
      var y = b[3].length;
      for (let A = 0; A < y; A += 10) A > 0 ? (w += "|" + String(A).padEnd(9)) : (w += "|" + String(1).padEnd(8));
      return [w];
    };
  return (
    document.getElementById("slide"),
    (l.$$set = (b) => {
      ("seq" in b && t(4, (s = b.seq)),
        "start" in b && t(0, (n = b.start)),
        "end" in b && t(1, (a = b.end)),
        "selstart" in b && t(5, (i = b.selstart)),
        "selend" in b && t(6, (u = b.selend)));
    }),
    (l.$$.update = () => {
      (l.$$.dirty & 16 && t(2, (d = v(s))),
        l.$$.dirty & 4 && t(3, (p = _(d))),
        l.$$.dirty & 1 && t(0, (n = Number(n))),
        l.$$.dirty & 1 && f(n),
        l.$$.dirty & 2 && t(1, (a = Number(a))));
    }),
    [n, a, d, p, s, i, u, f]
  );
}
class ja extends Dl {
  constructor(e) {
    (super(), Il(this, e, Oa, Da, Ll, { seq: 4, start: 0, end: 1, selstart: 5, selend: 6, setScrollPos: 7 }));
  }
  get setScrollPos() {
    return this.$$.ctx[7];
  }
}
function $i(l, e, t) {
  const s = l.slice();
  return ((s[255] = e[t]), s);
}
function er(l, e, t) {
  const s = l.slice();
  return ((s[255] = e[t]), (s[259] = t), s);
}
function tr(l, e, t) {
  const s = l.slice();
  return ((s[255] = e[t]), (s[259] = t), s);
}
function lr(l, e, t) {
  const s = l.slice();
  return ((s[260] = e[t]), s);
}
function sr(l, e, t) {
  const s = l.slice();
  return ((s[255] = e[t]), (s[259] = t), s);
}
function nr(l, e, t) {
  const s = l.slice();
  return ((s[255] = e[t]), (s[259] = t), s);
}
function ir(l, e, t) {
  const s = l.slice();
  return ((s[260] = e[t]), s);
}
function rr(l, e, t) {
  const s = l.slice();
  return ((s[260] = e[t]), s);
}
function or(l) {
  let e;
  return {
    c() {
      ((e = c("div")), (e.textContent = "Sequence length is too short - minimum length is 60 bp"), r(e, "class", "seqwarn svelte-1u6ecyq"));
    },
    m(t, s) {
      g(t, e, s);
    },
    d(t) {
      t && m(e);
    },
  };
}
function ar(l) {
  let e,
    t,
    s = jt.MAX_SEQ_LEN + "",
    n,
    a,
    i = l[0].length + "",
    u,
    f;
  return {
    c() {
      ((e = c("div")),
        (t = C("Sequence length too long - maximum length is ")),
        (n = C(s)),
        (a = C(" bp (got ")),
        (u = C(i)),
        (f = C(" bp) - primers were not built")),
        r(e, "class", "seqwarn svelte-1u6ecyq"));
    },
    m(h, d) {
      (g(h, e, d), o(e, t), o(e, n), o(e, a), o(e, u), o(e, f));
    },
    p(h, d) {
      d[0] & 1 && i !== (i = h[0].length + "") && Z(u, i);
    },
    d(h) {
      h && m(e);
    },
  };
}
function Ba(l) {
  let e, t, s, n, a, i, u, f, h, d, p, v, _, b, w, y, A, q, T, N, M, k, E, P, G, O, K, D, L;
  return {
    c() {
      ((e = C("Enter a DNA sequence by copying and pasting it into the sequence text area ")),
        (t = c("br")),
        (s = C(`
				or uploading a file from your computer. Acceptable formats for the DNA sequence are:`)),
        (n = c("br")),
        (a = S()),
        (i = c("br")),
        (u = C(`
				raw -- plain text without IDs and descriptions or numbers `)),
        (f = c("br")),
        (h = C(` 
				FASTA -- first line consisting of ">ID optional description" `)),
        (d = c("br")),
        (p = C(` 
				GenBank -- standard GenBank (LOCUS and ORIGIN lines expected). `)),
        (v = c("br")),
        (_ = S()),
        (b = c("br")),
        (w = C(`

				Selecting the "Use Reverse Complement" checkbox instructs NEBaseChanger to use `)),
        (y = c("br")),
        (A = C(`
				the reverse complement of the sequence entered in the sequence text area. `)),
        (q = c("br")),
        (T = C(`
				Selecting the "Partial" checkbox tells NEBaseChanger that it should not assume `)),
        (N = c("br")),
        (M = C(`
				a complete circular sequence, allowing the entry of a much shorter sequence.  `)),
        (k = c("br")),
        (E = C(`
				However, partial sequences will need to include sufficient sequence flanking the mutation site, `)),
        (P = c("br")),
        (G = C(`
				currently set at a minimum of 35 nt 5\u2032 and 25 nt 3\u2032. Mutations will be prohibited `)),
        (O = c("br")),
        (K = C(`
				from being too close to the ends of the provided partial sequence fragment `)),
        (D = c("br")),
        (L = C(`
				and cannot span the ends of the fragment.`)),
        r(t, "class", "svelte-1u6ecyq"),
        r(n, "class", "svelte-1u6ecyq"),
        r(i, "class", "svelte-1u6ecyq"),
        r(f, "class", "svelte-1u6ecyq"),
        r(d, "class", "svelte-1u6ecyq"),
        r(v, "class", "svelte-1u6ecyq"),
        r(b, "class", "svelte-1u6ecyq"),
        r(y, "class", "svelte-1u6ecyq"),
        r(q, "class", "svelte-1u6ecyq"),
        r(N, "class", "svelte-1u6ecyq"),
        r(k, "class", "svelte-1u6ecyq"),
        r(P, "class", "svelte-1u6ecyq"),
        r(O, "class", "svelte-1u6ecyq"),
        r(D, "class", "svelte-1u6ecyq"));
    },
    m(R, Q) {
      (g(R, e, Q),
        g(R, t, Q),
        g(R, s, Q),
        g(R, n, Q),
        g(R, a, Q),
        g(R, i, Q),
        g(R, u, Q),
        g(R, f, Q),
        g(R, h, Q),
        g(R, d, Q),
        g(R, p, Q),
        g(R, v, Q),
        g(R, _, Q),
        g(R, b, Q),
        g(R, w, Q),
        g(R, y, Q),
        g(R, A, Q),
        g(R, q, Q),
        g(R, T, Q),
        g(R, N, Q),
        g(R, M, Q),
        g(R, k, Q),
        g(R, E, Q),
        g(R, P, Q),
        g(R, G, Q),
        g(R, O, Q),
        g(R, K, Q),
        g(R, D, Q),
        g(R, L, Q));
    },
    p: qe,
    d(R) {
      (R && m(e),
        R && m(t),
        R && m(s),
        R && m(n),
        R && m(a),
        R && m(i),
        R && m(u),
        R && m(f),
        R && m(h),
        R && m(d),
        R && m(p),
        R && m(v),
        R && m(_),
        R && m(b),
        R && m(w),
        R && m(y),
        R && m(A),
        R && m(q),
        R && m(T),
        R && m(N),
        R && m(M),
        R && m(k),
        R && m(E),
        R && m(P),
        R && m(G),
        R && m(O),
        R && m(K),
        R && m(D),
        R && m(L));
    },
  };
}
function Ha(l) {
  let e, t, s, n, a, i, u, f, h, d, p, v, _, b, w, y, A, q, T, N, M;
  return {
    c() {
      ((e = C('After entering the starting plasmid sequence, choose either "AA Point Mutation", ')),
        (t = c("br")),
        (s = C(`
				"Indel/Substitution", "Batch", or "Legacy". Point mutation mode is most convenient `)),
        (n = c("br")),
        (a = C(`
				if the mutagenesis is limited to one or more single codons, whereas the general `)),
        (i = c("br")),
        (u = C(`
				Indel/Substitution mode works with larger (and smaller) single insertions and substitutions `)),
        (f = c("br")),
        (h = C(`
				and also allows deletions. Batch mode is designed for a longer list of mutations `)),
        (d = c("br")),
        (p = C(`
				and allows mixing of point and indel styles of entry. `)),
        (v = c("br")),
        (_ = S()),
        (b = c("br")),
        (w = C(`
				Legacy mode is intended to emulate NEBaseChanger V1 and produce results that are consistent `)),
        (y = c("br")),
        (A = C(`
				with the parameters used in V1. Principally, the minimum primer length is reset to 18 nt `)),
        (q = c("br")),
        (T = C(`
				and insertions are always in the 5\u2032 tails.  Legacy mode has a simpler interface `)),
        (N = c("br")),
        (M = C(`
				that will be familiar to users of V1, but it is limited in functionality.`)),
        r(t, "class", "svelte-1u6ecyq"),
        r(n, "class", "svelte-1u6ecyq"),
        r(i, "class", "svelte-1u6ecyq"),
        r(f, "class", "svelte-1u6ecyq"),
        r(d, "class", "svelte-1u6ecyq"),
        r(v, "class", "svelte-1u6ecyq"),
        r(b, "class", "svelte-1u6ecyq"),
        r(y, "class", "svelte-1u6ecyq"),
        r(q, "class", "svelte-1u6ecyq"),
        r(N, "class", "svelte-1u6ecyq"));
    },
    m(k, E) {
      (g(k, e, E),
        g(k, t, E),
        g(k, s, E),
        g(k, n, E),
        g(k, a, E),
        g(k, i, E),
        g(k, u, E),
        g(k, f, E),
        g(k, h, E),
        g(k, d, E),
        g(k, p, E),
        g(k, v, E),
        g(k, _, E),
        g(k, b, E),
        g(k, w, E),
        g(k, y, E),
        g(k, A, E),
        g(k, q, E),
        g(k, T, E),
        g(k, N, E),
        g(k, M, E));
    },
    p: qe,
    d(k) {
      (k && m(e),
        k && m(t),
        k && m(s),
        k && m(n),
        k && m(a),
        k && m(i),
        k && m(u),
        k && m(f),
        k && m(h),
        k && m(d),
        k && m(p),
        k && m(v),
        k && m(_),
        k && m(b),
        k && m(w),
        k && m(y),
        k && m(A),
        k && m(q),
        k && m(T),
        k && m(N),
        k && m(M));
    },
  };
}
function Ua(l) {
  let e,
    t,
    s,
    n,
    a,
    i,
    u,
    f,
    h,
    d,
    p,
    v,
    _,
    b,
    w,
    y,
    A,
    q,
    T,
    N,
    M,
    k,
    E,
    P,
    G,
    O,
    K,
    D,
    L,
    R,
    Q,
    ae,
    W,
    x,
    H,
    U,
    le,
    J,
    ie,
    z,
    te,
    pe,
    Y,
    ue,
    he,
    de,
    _e,
    ne,
    $,
    fe,
    we,
    j,
    F,
    ee;
  return {
    c() {
      ((e = C("LEGACY MODE ---")),
        (t = c("br")),
        (s = S()),
        (n = c("br")),
        (a = C(`
	            Legacy mode is intended to emulate NEBaseChanger V1 and produce results `)),
        (i = c("br")),
        (u = C(`
	            that are consistent with the parameters used in V1. Principally, the minimum `)),
        (f = c("br")),
        (h = C(`
	            primer length is reset to 18nt and insertions are always in the 5\u2032 tails.`)),
        (d = c("br")),
        (p = S()),
        (v = c("br")),
        (_ = C(`
		  		Insert/delete/substitute operations require a 5' `)),
        (b = c("i")),
        (b.textContent = "flanking"),
        (w = C(" position (start), ")),
        (y = c("br")),
        (A = C(`
		  		3' `)),
        (q = c("i")),
        (q.textContent = "flanking"),
        (T = C(" position (end); the target sequence to be mutatated is ")),
        (N = c("br")),
        (M = S()),
        (k = c("b")),
        (k.textContent = "between"),
        (E = C(" the start and end positions.")),
        (P = c("br")),
        (G = C("This differs from V1 where substitutions ")),
        (O = c("br")),
        (K = C(`
		  		and deletions were identified by the coordinates of the target itself. `)),
        (D = c("br")),
        (L = S()),
        (R = c("br")),
        (Q = C(`
		  		In addition to manualy setting the start/end coordinates of the target,`)),
        (ae = c("br")),
        (W = C(`
				the target sequence may be selected by click and drag (text selection) `)),
        (x = c("br")),
        (H = C(`
				in the DNA sequence row. For insertions, clicking between two bases will select the `)),
        (U = c("br")),
        (le = C(`
				proper flanking bases. The target sequence may also be located using the search field.`)),
        (J = c("br")),
        (ie = S()),
        (z = c("br")),
        (te = C(`
		  		For deletions, leave the replacement sequence blank.`)),
        (pe = c("br")),
        (Y = S()),
        (ue = c("br")),
        (he = C(`
		  		Ambiguous base codes can be entered in the replacement sequence (N,Y,S,W, etc) `)),
        (de = c("br")),
        (_e = C(`
		  		and will be expanded combinatorialy. `)),
        (ne = c("br")),
        ($ = S()),
        (fe = c("br")),
        (we = C(`
		  		For example, a replacement sequence containing NS will be expanded to 8 sequences `)),
        (j = c("br")),
        (F = C(`
		  		containing: AC, CC, GC, TC, AG, CG, GG, and TG, generating 8 primer pairs.`)),
        (ee = c("br")),
        r(t, "class", "svelte-1u6ecyq"),
        r(n, "class", "svelte-1u6ecyq"),
        r(i, "class", "svelte-1u6ecyq"),
        r(f, "class", "svelte-1u6ecyq"),
        r(d, "class", "svelte-1u6ecyq"),
        r(v, "class", "svelte-1u6ecyq"),
        r(b, "class", "svelte-1u6ecyq"),
        r(y, "class", "svelte-1u6ecyq"),
        r(q, "class", "svelte-1u6ecyq"),
        r(N, "class", "svelte-1u6ecyq"),
        r(k, "class", "svelte-1u6ecyq"),
        r(P, "class", "svelte-1u6ecyq"),
        r(O, "class", "svelte-1u6ecyq"),
        r(D, "class", "svelte-1u6ecyq"),
        r(R, "class", "svelte-1u6ecyq"),
        r(ae, "class", "svelte-1u6ecyq"),
        r(x, "class", "svelte-1u6ecyq"),
        r(U, "class", "svelte-1u6ecyq"),
        r(J, "class", "svelte-1u6ecyq"),
        r(z, "class", "svelte-1u6ecyq"),
        r(pe, "class", "svelte-1u6ecyq"),
        r(ue, "class", "svelte-1u6ecyq"),
        r(de, "class", "svelte-1u6ecyq"),
        r(ne, "class", "svelte-1u6ecyq"),
        r(fe, "class", "svelte-1u6ecyq"),
        r(j, "class", "svelte-1u6ecyq"),
        r(ee, "class", "svelte-1u6ecyq"));
    },
    m(V, me) {
      (g(V, e, me),
        g(V, t, me),
        g(V, s, me),
        g(V, n, me),
        g(V, a, me),
        g(V, i, me),
        g(V, u, me),
        g(V, f, me),
        g(V, h, me),
        g(V, d, me),
        g(V, p, me),
        g(V, v, me),
        g(V, _, me),
        g(V, b, me),
        g(V, w, me),
        g(V, y, me),
        g(V, A, me),
        g(V, q, me),
        g(V, T, me),
        g(V, N, me),
        g(V, M, me),
        g(V, k, me),
        g(V, E, me),
        g(V, P, me),
        g(V, G, me),
        g(V, O, me),
        g(V, K, me),
        g(V, D, me),
        g(V, L, me),
        g(V, R, me),
        g(V, Q, me),
        g(V, ae, me),
        g(V, W, me),
        g(V, x, me),
        g(V, H, me),
        g(V, U, me),
        g(V, le, me),
        g(V, J, me),
        g(V, ie, me),
        g(V, z, me),
        g(V, te, me),
        g(V, pe, me),
        g(V, Y, me),
        g(V, ue, me),
        g(V, he, me),
        g(V, de, me),
        g(V, _e, me),
        g(V, ne, me),
        g(V, $, me),
        g(V, fe, me),
        g(V, we, me),
        g(V, j, me),
        g(V, F, me),
        g(V, ee, me));
    },
    p: qe,
    d(V) {
      (V && m(e),
        V && m(t),
        V && m(s),
        V && m(n),
        V && m(a),
        V && m(i),
        V && m(u),
        V && m(f),
        V && m(h),
        V && m(d),
        V && m(p),
        V && m(v),
        V && m(_),
        V && m(b),
        V && m(w),
        V && m(y),
        V && m(A),
        V && m(q),
        V && m(T),
        V && m(N),
        V && m(M),
        V && m(k),
        V && m(E),
        V && m(P),
        V && m(G),
        V && m(O),
        V && m(K),
        V && m(D),
        V && m(L),
        V && m(R),
        V && m(Q),
        V && m(ae),
        V && m(W),
        V && m(x),
        V && m(H),
        V && m(U),
        V && m(le),
        V && m(J),
        V && m(ie),
        V && m(z),
        V && m(te),
        V && m(pe),
        V && m(Y),
        V && m(ue),
        V && m(he),
        V && m(de),
        V && m(_e),
        V && m(ne),
        V && m($),
        V && m(fe),
        V && m(we),
        V && m(j),
        V && m(F),
        V && m(ee));
    },
  };
}
function Va(l) {
  let e, t, s, n, a, i, u, f, h, d, p, v, _, b, w, y, A, q, T, N, M, k, E, P, G, O, K, D, L, R, Q, ae;
  return {
    c() {
      ((e = C("Insert/delete/substitute (indel) operations require a 5' flanking position (start), ")),
        (t = c("br")),
        (s = C(`
		  		3' flanking position (end), and a replacement sequence. The target sequence to be `)),
        (n = c("br")),
        (a = C(`
		  		replaced is `)),
        (i = c("b")),
        (i.textContent = "between"),
        (u = C(" the start and end positions.")),
        (f = c("br")),
        (h = c("br")),
        (d = C(`
		  		To delete a sequence, leave the replacement sequence blank.`)),
        (p = c("br")),
        (v = c("br")),
        (_ = C(`
		  		To insert a sequence, the start and end should be consecutive positions.`)),
        (b = c("br")),
        (w = C(` 
		  		The replacement sequence will be inserted between them. `)),
        (y = c("br")),
        (A = c("br")),
        (q = C(`
		  		For example, if start/end is 234/239 and the replacement sequence is ACGT, `)),
        (T = c("br")),
        (N = C(`
		  		the operation will substitute the sequence between bases 234 and 239 with ACGT. `)),
        (M = c("br")),
        (k = S()),
        (E = c("br")),
        (P = C(`
		  		Ambiguous base codes can be entered in the replacement sequence (N,Y,S,W, etc) `)),
        (G = c("br")),
        (O = C(`
		  		and will be expanded combinatorialy. `)),
        (K = c("br")),
        (D = c("br")),
        (L = C(`
		  		For example, a replacement sequence containing NS will be expanded to 8 sequences `)),
        (R = c("br")),
        (Q = C(`
		  		containing: AC, CC, GC, TC, AG, CG, GG, and TG, generating 8 primer pairs.`)),
        (ae = c("br")),
        r(t, "class", "svelte-1u6ecyq"),
        r(n, "class", "svelte-1u6ecyq"),
        r(i, "class", "svelte-1u6ecyq"),
        r(f, "class", "svelte-1u6ecyq"),
        r(h, "class", "svelte-1u6ecyq"),
        r(p, "class", "svelte-1u6ecyq"),
        r(v, "class", "svelte-1u6ecyq"),
        r(b, "class", "svelte-1u6ecyq"),
        r(y, "class", "svelte-1u6ecyq"),
        r(A, "class", "svelte-1u6ecyq"),
        r(T, "class", "svelte-1u6ecyq"),
        r(M, "class", "svelte-1u6ecyq"),
        r(E, "class", "svelte-1u6ecyq"),
        r(G, "class", "svelte-1u6ecyq"),
        r(K, "class", "svelte-1u6ecyq"),
        r(D, "class", "svelte-1u6ecyq"),
        r(R, "class", "svelte-1u6ecyq"),
        r(ae, "class", "svelte-1u6ecyq"));
    },
    m(W, x) {
      (g(W, e, x),
        g(W, t, x),
        g(W, s, x),
        g(W, n, x),
        g(W, a, x),
        g(W, i, x),
        g(W, u, x),
        g(W, f, x),
        g(W, h, x),
        g(W, d, x),
        g(W, p, x),
        g(W, v, x),
        g(W, _, x),
        g(W, b, x),
        g(W, w, x),
        g(W, y, x),
        g(W, A, x),
        g(W, q, x),
        g(W, T, x),
        g(W, N, x),
        g(W, M, x),
        g(W, k, x),
        g(W, E, x),
        g(W, P, x),
        g(W, G, x),
        g(W, O, x),
        g(W, K, x),
        g(W, D, x),
        g(W, L, x),
        g(W, R, x),
        g(W, Q, x),
        g(W, ae, x));
    },
    p: qe,
    d(W) {
      (W && m(e),
        W && m(t),
        W && m(s),
        W && m(n),
        W && m(a),
        W && m(i),
        W && m(u),
        W && m(f),
        W && m(h),
        W && m(d),
        W && m(p),
        W && m(v),
        W && m(_),
        W && m(b),
        W && m(w),
        W && m(y),
        W && m(A),
        W && m(q),
        W && m(T),
        W && m(N),
        W && m(M),
        W && m(k),
        W && m(E),
        W && m(P),
        W && m(G),
        W && m(O),
        W && m(K),
        W && m(D),
        W && m(L),
        W && m(R),
        W && m(Q),
        W && m(ae));
    },
  };
}
function Ka(l) {
  let e, t, s, n, a, i, u, f, h, d, p, v, _, b, w, y, A, q, T, N, M, k, E, P, G, O, K, D, L, R, Q, ae, W, x;
  return {
    c() {
      ((e = C("First, enter the start position of the ORF, either directly by entering the coordinate")),
        (t = c("br")),
        (s = C(`
	    		or by clicking in the DNA sequence row of the codon view. Setting the ORF start `)),
        (n = c("br")),
        (a = C(`
	    		position will highlight the ORF. Changing it keeps the mutations you've entered; `)),
        (i = c("br")),
        (u = C(`
	    		any that no longer match the new frame are skipped, with a warning.`)),
        (f = c("br")),
        (h = S()),
        (d = c("br")),
        (p = C(`
	    		Next, enter one or more mutations of the form [old AA][codon #][new AA], `)),
        (v = c("br")),
        (_ = C(`
	    		for example, M2A, F32H, C45Y, etc. The codon # is relative to the selected `)),
        (b = c("br")),
        (w = C(`
	    		ORF start.  Optionally, specify the exact triplet replacement by appending `)),
        (y = c("br")),
        (A = C(`
	    		a colon followed by the triplet, for example, M2G:GGG, A34F:TTC. Exact triplets, `)),
        (q = c("br")),
        (T = C(`
	    		will override automatic selection. Multiple mutations can be entered, separated by spaces, commas or semicolons.`)),
        (N = c("br")),
        (M = S()),
        (k = c("br")),
        (E = C(`
	    		Clicking on an amino acid in the codon view will initiate a dialog to select the replacement`)),
        (P = c("br")),
        (G = C(`
	    		amino acid, including the exact triplet, and will add it to the list of mutations.`)),
        (O = c("br")),
        (K = S()),
        (D = c("br")),
        (L = C(`
	    		The old AA is checked, but processing is not halted if the old AA does not match`)),
        (R = c("br")),
        (Q = C(`
	    		the indicated codon #. The codon number takes presidence. Also, the ORF start`)),
        (ae = c("br")),
        (W = C(` 
	    		can be set arbitrarily and is not checked for validity.`)),
        (x = c("br")),
        r(t, "class", "svelte-1u6ecyq"),
        r(n, "class", "svelte-1u6ecyq"),
        r(i, "class", "svelte-1u6ecyq"),
        r(f, "class", "svelte-1u6ecyq"),
        r(d, "class", "svelte-1u6ecyq"),
        r(v, "class", "svelte-1u6ecyq"),
        r(b, "class", "svelte-1u6ecyq"),
        r(y, "class", "svelte-1u6ecyq"),
        r(q, "class", "svelte-1u6ecyq"),
        r(N, "class", "svelte-1u6ecyq"),
        r(k, "class", "svelte-1u6ecyq"),
        r(P, "class", "svelte-1u6ecyq"),
        r(O, "class", "svelte-1u6ecyq"),
        r(D, "class", "svelte-1u6ecyq"),
        r(R, "class", "svelte-1u6ecyq"),
        r(ae, "class", "svelte-1u6ecyq"),
        r(x, "class", "svelte-1u6ecyq"));
    },
    m(H, U) {
      (g(H, e, U),
        g(H, t, U),
        g(H, s, U),
        g(H, n, U),
        g(H, a, U),
        g(H, i, U),
        g(H, u, U),
        g(H, f, U),
        g(H, h, U),
        g(H, d, U),
        g(H, p, U),
        g(H, v, U),
        g(H, _, U),
        g(H, b, U),
        g(H, w, U),
        g(H, y, U),
        g(H, A, U),
        g(H, q, U),
        g(H, T, U),
        g(H, N, U),
        g(H, M, U),
        g(H, k, U),
        g(H, E, U),
        g(H, P, U),
        g(H, G, U),
        g(H, O, U),
        g(H, K, U),
        g(H, D, U),
        g(H, L, U),
        g(H, R, U),
        g(H, Q, U),
        g(H, ae, U),
        g(H, W, U),
        g(H, x, U));
    },
    p: qe,
    d(H) {
      (H && m(e),
        H && m(t),
        H && m(s),
        H && m(n),
        H && m(a),
        H && m(i),
        H && m(u),
        H && m(f),
        H && m(h),
        H && m(d),
        H && m(p),
        H && m(v),
        H && m(_),
        H && m(b),
        H && m(w),
        H && m(y),
        H && m(A),
        H && m(q),
        H && m(T),
        H && m(N),
        H && m(M),
        H && m(k),
        H && m(E),
        H && m(P),
        H && m(G),
        H && m(O),
        H && m(K),
        H && m(D),
        H && m(L),
        H && m(R),
        H && m(Q),
        H && m(ae),
        H && m(W),
        H && m(x));
    },
  };
}
function Ya(l) {
  let e,
    t,
    s,
    n,
    a,
    i,
    u,
    f,
    h,
    d,
    p,
    v,
    _,
    b,
    w,
    y,
    A,
    q,
    T,
    N,
    M,
    k,
    E,
    P,
    G,
    O,
    K,
    D,
    L,
    R,
    Q,
    ae,
    W,
    x,
    H,
    U,
    le,
    J,
    ie,
    z,
    te,
    pe,
    Y,
    ue,
    he,
    de,
    _e,
    ne,
    $,
    fe,
    we,
    j;
  return {
    c() {
      ((e = C("Batch input mode allows multiple command line style operations to be applied ")),
        (t = c("br")),
        (s = C(`
	    		to the currently loaded sequence. It is intended to facilitate the entry `)),
        (n = c("br")),
        (a = C(`
	    		of large numbers of mutations on a single input sequence.`)),
        (i = c("br")),
        (u = S()),
        (f = c("br")),
        (h = C(`
	    		Batch commands can be loaded from a text file, pasted into the text area, or typed in directly. `)),
        (d = c("br")),
        (p = S()),
        (v = c("br")),
        (_ = C(`
	    		All batch input operates on the currently loaded sequence. Each batch command `)),
        (b = c("br")),
        (w = C(`
	    		should be on a separate line. The following commands are supported in Batch mode: `)),
        (y = c("br")),
        (A = C(`
	    		1. ORF: [start]`)),
        (q = c("br")),
        (T = C(`
	    		2. [old AA][codon #][new AA]:[triplet]`)),
        (N = c("br")),
        (M = C(`
	    		3. [ID] [start] [end] [replacement]`)),
        (k = c("br")),
        (E = S()),
        (P = c("br")),
        (G = S()),
        (O = c("dt")),
        (O.textContent = "ORF: [start]"),
        (K = S()),
        (D = c("dd")),
        (D.innerHTML = `Specifies the coordinate of the first base of the ORF. Required for subsequent AA point mutations. <br class="svelte-1u6ecyq"/> 
		            <br class="svelte-1u6ecyq"/>
	    			Examples:<br class="svelte-1u6ecyq"/>
	    			ORF: 1256<br class="svelte-1u6ecyq"/>
	    			ORF: 2<br class="svelte-1u6ecyq"/> 
	    			<br class="svelte-1u6ecyq"/>
	    			[start] must be a positive number in the range 1 to sequence length. <br class="svelte-1u6ecyq"/>
	    			All subsequent point mutation commands will be relative to the specified ORF start.<br class="svelte-1u6ecyq"/> 
	    			<br class="svelte-1u6ecyq"/><br class="svelte-1u6ecyq"/>`),
        (L = S()),
        (R = c("dt")),
        (R.textContent = "[old AA][codon #][new AA]:[triplet]"),
        (Q = S()),
        (ae = c("dd")),
        (ae.innerHTML = `Specifies an AA point mutation at a specific codon.<br class="svelte-1u6ecyq"/> 
	    			<br class="svelte-1u6ecyq"/>
	    			Examples:<br class="svelte-1u6ecyq"/>
	    			A2F<br class="svelte-1u6ecyq"/>
	    			A2F:TTT<br class="svelte-1u6ecyq"/> 
	    			<br class="svelte-1u6ecyq"/>
	    			[Old AA] and [new AA] are single letter codes for the current and mutated AA, respectively.<br class="svelte-1u6ecyq"/> 
	    			[codon #] counts triplets from the last specified ORF start coordinate. <br class="svelte-1u6ecyq"/>
	    			The :[triplet] specifier is optional. If provided, it should consist of <br class="svelte-1u6ecyq"/>
	    			a colon followed immediately by the triplet replacement sequence. If not specified,<br class="svelte-1u6ecyq"/> 
	    			(leave off the colon) the replacement triplet will be the codon with the highest usage <br class="svelte-1u6ecyq"/>
	    			in <i class="svelte-1u6ecyq">E. coli</i> K12. Exact triplets, will override automatic selection.<br class="svelte-1u6ecyq"/> 
	    			<br class="svelte-1u6ecyq"/><br class="svelte-1u6ecyq"/>`),
        (W = S()),
        (x = c("dt")),
        (x.textContent = "[ID] [start] [end] [replacement]"),
        (H = S()),
        (U = c("dd")),
        (U.innerHTML = `Specifies a general indel/substitution operation. <br class="svelte-1u6ecyq"/> 
					<br class="svelte-1u6ecyq"/>
	    		    Examples:<br class="svelte-1u6ecyq"/>
	    			ID1 234 239 <i class="svelte-1u6ecyq">deletes</i> the sequence <i class="svelte-1u6ecyq">between</i> bases 234 and 239. <br class="svelte-1u6ecyq"/>
	    			ID2 234 239 ACGT <i class="svelte-1u6ecyq">substitutes</i> ACGT for the sequence <i class="svelte-1u6ecyq">between</i> bases 234 and 239. <br class="svelte-1u6ecyq"/>
	    			ID3 234 235 ACGT <i class="svelte-1u6ecyq">inserts</i> ACGT <i class="svelte-1u6ecyq">between</i> bases 234 and 235. <br class="svelte-1u6ecyq"/> 
	    			<br class="svelte-1u6ecyq"/>
					[Start] and [end] are the base positions <i class="svelte-1u6ecyq">flanking</i> the target sequence <br class="svelte-1u6ecyq"/>
					to be replaced. If [replacement] is omitted, the target sequence will be deleted.<br class="svelte-1u6ecyq"/>  
					To insert a sequence, [start] and [end] should be consecutive positions.<br class="svelte-1u6ecyq"/> 
	    		    <br class="svelte-1u6ecyq"/><br class="svelte-1u6ecyq"/>`),
        (le = C(`
	    		Any ambiguous base codes (N,Y,S,W, etc) will be expanded combinatorialy `)),
        (J = c("br")),
        (ie = C(`
	    		unless "Preserve degenerate bases" is checked.`)),
        (z = c("br")),
        (te = C(`
	    		For example, a replacement sequence containing NS will be expanded to 8 sequences `)),
        (pe = c("br")),
        (Y = C(`
	    		containing: AC, CC, GC, TC, AG, CG, GG, and TG, generating 8 primer pairs.`)),
        (ue = c("br")),
        (he = S()),
        (de = c("br")),
        (_e = C(`
	    		The following is an example of a valid set of batch commands:`)),
        (ne = c("br")),
        ($ = S()),
        (fe = c("br")),
        (we = S()),
        (j = c("b")),
        (j.innerHTML = `ORF: 100<br class="svelte-1u6ecyq"/>
					M1Y<br class="svelte-1u6ecyq"/>
					A42S<br class="svelte-1u6ecyq"/>
					P24F:TTT<br class="svelte-1u6ecyq"/>
					setA 200 205 NNA<br class="svelte-1u6ecyq"/>
					InSetB 400 401 AACGGTGTN<br class="svelte-1u6ecyq"/>
					Site1 100 105 ACGT<br class="svelte-1u6ecyq"/>
					D1 1000 1200 <br class="svelte-1u6ecyq"/>
					setE 1800 200 ATNCG<br class="svelte-1u6ecyq"/>`),
        r(t, "class", "svelte-1u6ecyq"),
        r(n, "class", "svelte-1u6ecyq"),
        r(i, "class", "svelte-1u6ecyq"),
        r(f, "class", "svelte-1u6ecyq"),
        r(d, "class", "svelte-1u6ecyq"),
        r(v, "class", "svelte-1u6ecyq"),
        r(b, "class", "svelte-1u6ecyq"),
        r(y, "class", "svelte-1u6ecyq"),
        r(q, "class", "svelte-1u6ecyq"),
        r(N, "class", "svelte-1u6ecyq"),
        r(k, "class", "svelte-1u6ecyq"),
        r(P, "class", "svelte-1u6ecyq"),
        r(O, "class", "svelte-1u6ecyq"),
        r(D, "class", "svelte-1u6ecyq"),
        r(R, "class", "svelte-1u6ecyq"),
        r(ae, "class", "svelte-1u6ecyq"),
        r(x, "class", "svelte-1u6ecyq"),
        r(U, "class", "svelte-1u6ecyq"),
        r(J, "class", "svelte-1u6ecyq"),
        r(z, "class", "svelte-1u6ecyq"),
        r(pe, "class", "svelte-1u6ecyq"),
        r(ue, "class", "svelte-1u6ecyq"),
        r(de, "class", "svelte-1u6ecyq"),
        r(ne, "class", "svelte-1u6ecyq"),
        r(fe, "class", "svelte-1u6ecyq"),
        r(j, "class", "svelte-1u6ecyq"));
    },
    m(F, ee) {
      (g(F, e, ee),
        g(F, t, ee),
        g(F, s, ee),
        g(F, n, ee),
        g(F, a, ee),
        g(F, i, ee),
        g(F, u, ee),
        g(F, f, ee),
        g(F, h, ee),
        g(F, d, ee),
        g(F, p, ee),
        g(F, v, ee),
        g(F, _, ee),
        g(F, b, ee),
        g(F, w, ee),
        g(F, y, ee),
        g(F, A, ee),
        g(F, q, ee),
        g(F, T, ee),
        g(F, N, ee),
        g(F, M, ee),
        g(F, k, ee),
        g(F, E, ee),
        g(F, P, ee),
        g(F, G, ee),
        g(F, O, ee),
        g(F, K, ee),
        g(F, D, ee),
        g(F, L, ee),
        g(F, R, ee),
        g(F, Q, ee),
        g(F, ae, ee),
        g(F, W, ee),
        g(F, x, ee),
        g(F, H, ee),
        g(F, U, ee),
        g(F, le, ee),
        g(F, J, ee),
        g(F, ie, ee),
        g(F, z, ee),
        g(F, te, ee),
        g(F, pe, ee),
        g(F, Y, ee),
        g(F, ue, ee),
        g(F, he, ee),
        g(F, de, ee),
        g(F, _e, ee),
        g(F, ne, ee),
        g(F, $, ee),
        g(F, fe, ee),
        g(F, we, ee),
        g(F, j, ee));
    },
    p: qe,
    d(F) {
      (F && m(e),
        F && m(t),
        F && m(s),
        F && m(n),
        F && m(a),
        F && m(i),
        F && m(u),
        F && m(f),
        F && m(h),
        F && m(d),
        F && m(p),
        F && m(v),
        F && m(_),
        F && m(b),
        F && m(w),
        F && m(y),
        F && m(A),
        F && m(q),
        F && m(T),
        F && m(N),
        F && m(M),
        F && m(k),
        F && m(E),
        F && m(P),
        F && m(G),
        F && m(O),
        F && m(K),
        F && m(D),
        F && m(L),
        F && m(R),
        F && m(Q),
        F && m(ae),
        F && m(W),
        F && m(x),
        F && m(H),
        F && m(U),
        F && m(le),
        F && m(J),
        F && m(ie),
        F && m(z),
        F && m(te),
        F && m(pe),
        F && m(Y),
        F && m(ue),
        F && m(he),
        F && m(de),
        F && m(_e),
        F && m(ne),
        F && m($),
        F && m(fe),
        F && m(we),
        F && m(j));
    },
  };
}
function Wa(l) {
  let e, t, s, n, a, i, u, f, h, d, p, v, _, b, w, y, A, q, T, N, M, k, E, P;
  return {
    c() {
      ((e = C("All batch input operates on the currently loaded sequence. Each batch command ")),
        (t = c("br")),
        (s = C(`
	    		should be on a separate line. The following commands are supported in Batch mode: `)),
        (n = c("br")),
        (a = C(`
	    		1. ORF: [start]`)),
        (i = c("br")),
        (u = C(`
	    		2. [old AA][codon #][new AA]:[triplet]`)),
        (f = c("br")),
        (h = C(`
	    		3. [ID] [start] [end] [replacement]`)),
        (d = c("br")),
        (p = S()),
        (v = c("br")),
        (_ = S()),
        (b = c("dt")),
        (b.textContent = "ORF: [start]"),
        (w = S()),
        (y = c("dd")),
        (y.innerHTML = `Specifies the coordinate of the first base of the ORF. Required for subsequent AA point mutations. <br class="svelte-1u6ecyq"/> 
		            <br class="svelte-1u6ecyq"/>
	    			Examples:<br class="svelte-1u6ecyq"/>
	    			ORF: 1256<br class="svelte-1u6ecyq"/>
	    			ORF: 2<br class="svelte-1u6ecyq"/> 
	    			<br class="svelte-1u6ecyq"/>
	    			[start] must be a positive number in the range 1 to sequence length. <br class="svelte-1u6ecyq"/>
	    			All subsequent point mutation commands will be relative to the specified ORF start.<br class="svelte-1u6ecyq"/> 
	    			<br class="svelte-1u6ecyq"/><br class="svelte-1u6ecyq"/>`),
        (A = S()),
        (q = c("dt")),
        (q.textContent = "[old AA][codon #][new AA]:[triplet]"),
        (T = S()),
        (N = c("dd")),
        (N.innerHTML = `Specifies an AA point mutation at a specific codon.<br class="svelte-1u6ecyq"/> 
	    			<br class="svelte-1u6ecyq"/>
	    			Examples:<br class="svelte-1u6ecyq"/>
	    			A2F<br class="svelte-1u6ecyq"/>
	    			A2F:TTT<br class="svelte-1u6ecyq"/> 
	    			<br class="svelte-1u6ecyq"/>
	    			[Old AA] and [new AA] are single letter codes for the current and mutated AA, respectively.<br class="svelte-1u6ecyq"/> 
	    			[codon #] counts triplets from the last specified ORF start coordinate. <br class="svelte-1u6ecyq"/>
	    			The :[triplet] specifier is optional. If provided, it should consist of <br class="svelte-1u6ecyq"/>
	    			a colon followed immediately by the triplet replacement sequence. If not specified,<br class="svelte-1u6ecyq"/> 
	    			(leave off the colon) the replacement triplet will be the codon with the highest usage <br class="svelte-1u6ecyq"/>
	    			in <i class="svelte-1u6ecyq">E. coli</i> K12. Exact triplets, will override automatic selection.<br class="svelte-1u6ecyq"/> 
	    			<br class="svelte-1u6ecyq"/><br class="svelte-1u6ecyq"/>`),
        (M = S()),
        (k = c("dt")),
        (k.textContent = "[ID] [start] [end] [replacement]"),
        (E = S()),
        (P = c("dd")),
        (P.innerHTML = `Specifies a general indel/substitution operation. <br class="svelte-1u6ecyq"/> 
					<br class="svelte-1u6ecyq"/>
	    		    Examples:<br class="svelte-1u6ecyq"/>
	    			ID1 234 239 <i class="svelte-1u6ecyq">deletes</i> the sequence <i class="svelte-1u6ecyq">between</i> bases 234 and 239. <br class="svelte-1u6ecyq"/>
	    			ID2 234 239 ACGT <i class="svelte-1u6ecyq">substitutes</i> ACGT for the sequence <i class="svelte-1u6ecyq">between</i> bases 234 and 239. <br class="svelte-1u6ecyq"/>
	    			ID3 234 235 ACGT <i class="svelte-1u6ecyq">inserts</i> ACGT <i class="svelte-1u6ecyq">between</i> bases 234 and 235. <br class="svelte-1u6ecyq"/> 
	    			<br class="svelte-1u6ecyq"/>
					[Start] and [end] are the base positions <i class="svelte-1u6ecyq">flanking</i> the target sequence <br class="svelte-1u6ecyq"/>
					to be replaced. If [replacement] is omitted, the target sequence will be deleted.<br class="svelte-1u6ecyq"/>  
					To insert a sequence, [start] and [end] should be consecutive positions.<br class="svelte-1u6ecyq"/> 
	    		    <br class="svelte-1u6ecyq"/><br class="svelte-1u6ecyq"/>`),
        r(t, "class", "svelte-1u6ecyq"),
        r(n, "class", "svelte-1u6ecyq"),
        r(i, "class", "svelte-1u6ecyq"),
        r(f, "class", "svelte-1u6ecyq"),
        r(d, "class", "svelte-1u6ecyq"),
        r(v, "class", "svelte-1u6ecyq"),
        r(b, "class", "svelte-1u6ecyq"),
        r(y, "class", "svelte-1u6ecyq"),
        r(q, "class", "svelte-1u6ecyq"),
        r(N, "class", "svelte-1u6ecyq"),
        r(k, "class", "svelte-1u6ecyq"),
        r(P, "class", "svelte-1u6ecyq"));
    },
    m(G, O) {
      (g(G, e, O),
        g(G, t, O),
        g(G, s, O),
        g(G, n, O),
        g(G, a, O),
        g(G, i, O),
        g(G, u, O),
        g(G, f, O),
        g(G, h, O),
        g(G, d, O),
        g(G, p, O),
        g(G, v, O),
        g(G, _, O),
        g(G, b, O),
        g(G, w, O),
        g(G, y, O),
        g(G, A, O),
        g(G, q, O),
        g(G, T, O),
        g(G, N, O),
        g(G, M, O),
        g(G, k, O),
        g(G, E, O),
        g(G, P, O));
    },
    p: qe,
    d(G) {
      (G && m(e),
        G && m(t),
        G && m(s),
        G && m(n),
        G && m(a),
        G && m(i),
        G && m(u),
        G && m(f),
        G && m(h),
        G && m(d),
        G && m(p),
        G && m(v),
        G && m(_),
        G && m(b),
        G && m(w),
        G && m(y),
        G && m(A),
        G && m(q),
        G && m(T),
        G && m(N),
        G && m(M),
        G && m(k),
        G && m(E),
        G && m(P));
    },
  };
}
function za(l) {
  let e, t, s, n, a, i, u, f, h, d, p, v, _, b, w, y, A, q, T, N, M;
  return {
    c() {
      ((e = C('If the "Confine mutations to primer 5\u2032 tails" option is selected, ')),
        (t = c("br")),
        (s = C(`
		  		the replacement sequence will always be attached to the 5\u2032 end `)),
        (n = c("br")),
        (a = C(`
		  		of the primers, regardless of the length of the substitution. `)),
        (i = c("br")),
        (u = C(`
		  		Normally, for substitutions, if the target region is longer than 6 nucleotides, `)),
        (f = c("br")),
        (h = C(`
		  		the substitution sequence will be split between the two primers, attached `)),
        (d = c("br")),
        (p = C(`
		  		to the 5\u2032 end of the primers as non-annealing tails. The 5\u2032 ends `)),
        (v = c("br")),
        (_ = C(`
		  		of the annealing portions of the primers align with the edges of the target region. `)),
        (b = c("br")),
        (w = C(`
		  		If the target region encompasses 6 or fewer nucleotides, the replacement sequence `)),
        (y = c("br")),
        (A = C(`
		  		will be placed in the forward primer with 10 or more bases of annealing sequence `)),
        (q = c("br")),
        (T = C(`
		  		flanking the inserted bases. `)),
        (N = c("br")),
        (M = C(`
		  		This option is always on while "Preserve degenerate bases" is selected.`)),
        r(t, "class", "svelte-1u6ecyq"),
        r(n, "class", "svelte-1u6ecyq"),
        r(i, "class", "svelte-1u6ecyq"),
        r(f, "class", "svelte-1u6ecyq"),
        r(d, "class", "svelte-1u6ecyq"),
        r(v, "class", "svelte-1u6ecyq"),
        r(b, "class", "svelte-1u6ecyq"),
        r(y, "class", "svelte-1u6ecyq"),
        r(q, "class", "svelte-1u6ecyq"),
        r(N, "class", "svelte-1u6ecyq"));
    },
    m(k, E) {
      (g(k, e, E),
        g(k, t, E),
        g(k, s, E),
        g(k, n, E),
        g(k, a, E),
        g(k, i, E),
        g(k, u, E),
        g(k, f, E),
        g(k, h, E),
        g(k, d, E),
        g(k, p, E),
        g(k, v, E),
        g(k, _, E),
        g(k, b, E),
        g(k, w, E),
        g(k, y, E),
        g(k, A, E),
        g(k, q, E),
        g(k, T, E),
        g(k, N, E),
        g(k, M, E));
    },
    p: qe,
    d(k) {
      (k && m(e),
        k && m(t),
        k && m(s),
        k && m(n),
        k && m(a),
        k && m(i),
        k && m(u),
        k && m(f),
        k && m(h),
        k && m(d),
        k && m(p),
        k && m(v),
        k && m(_),
        k && m(b),
        k && m(w),
        k && m(y),
        k && m(A),
        k && m(q),
        k && m(T),
        k && m(N),
        k && m(M));
    },
  };
}
function Xa(l) {
  let e, t, s, n, a, i, u, f, h;
  return {
    c() {
      ((e = C("This option keeps degenerate/ambiguous base codes in the primer sequence when selecting ")),
        (t = c("br")),
        (s = C(`
		  		the amino acid "X" representing all possible amino acids, in targeted saturation mutagenesis.  `)),
        (n = c("br")),
        (a = C(`
		  		Normally, the ambiguous bases (NNK) are expanded to create 32 primer pairs with only canonical bases. `)),
        (i = c("br")),
        (u = C(`
		  		With this option selected, only one primer pair will be generated containing "NNK". `)),
        (f = c("br")),
        (h = C(`
		  		This option is currently only compatible with mutations confined to the 5\u2032 primer tail.`)),
        r(t, "class", "svelte-1u6ecyq"),
        r(n, "class", "svelte-1u6ecyq"),
        r(i, "class", "svelte-1u6ecyq"),
        r(f, "class", "svelte-1u6ecyq"));
    },
    m(d, p) {
      (g(d, e, p), g(d, t, p), g(d, s, p), g(d, n, p), g(d, a, p), g(d, i, p), g(d, u, p), g(d, f, p), g(d, h, p));
    },
    p: qe,
    d(d) {
      (d && m(e), d && m(t), d && m(s), d && m(n), d && m(a), d && m(i), d && m(u), d && m(f), d && m(h));
    },
  };
}
function Qa(l) {
  let e, t, s, n, a, i, u, f, h, d, p, v, _, b, w, y, A;
  return {
    c() {
      ((e = C("This option merges point mutations to keep them within a single primer pair.")),
        (t = c("br")),
        (s = C(`
		  		Mutations are entered normally in the form [old AA][codon #][new AA], `)),
        (n = c("br")),
        (a = C(`
	    		for example, M1A, D3H, C4Y, etc., separated by spaces, commas or semicolons.`)),
        (i = c("br")),
        (u = S()),
        (f = c("br")),
        (h = C(`
		  		If the mutations are close together, generally not separated by more than 6 bases, `)),
        (d = c("br")),
        (p = C(`
		  		the mutations are merged and treated as a sequence replacement operation. `)),
        (v = c("br")),
        (_ = C(`
		  		In the example above, all 3 mutations would be embedded in a single primer pair.`)),
        (b = c("br")),
        (w = S()),
        (y = c("br")),
        (A = C(`
		  		This option is currently only compatible with mutations confined to the 5\u2032 primer tail.`)),
        r(t, "class", "svelte-1u6ecyq"),
        r(n, "class", "svelte-1u6ecyq"),
        r(i, "class", "svelte-1u6ecyq"),
        r(f, "class", "svelte-1u6ecyq"),
        r(d, "class", "svelte-1u6ecyq"),
        r(v, "class", "svelte-1u6ecyq"),
        r(b, "class", "svelte-1u6ecyq"),
        r(y, "class", "svelte-1u6ecyq"));
    },
    m(q, T) {
      (g(q, e, T),
        g(q, t, T),
        g(q, s, T),
        g(q, n, T),
        g(q, a, T),
        g(q, i, T),
        g(q, u, T),
        g(q, f, T),
        g(q, h, T),
        g(q, d, T),
        g(q, p, T),
        g(q, v, T),
        g(q, _, T),
        g(q, b, T),
        g(q, w, T),
        g(q, y, T),
        g(q, A, T));
    },
    p: qe,
    d(q) {
      (q && m(e),
        q && m(t),
        q && m(s),
        q && m(n),
        q && m(a),
        q && m(i),
        q && m(u),
        q && m(f),
        q && m(h),
        q && m(d),
        q && m(p),
        q && m(v),
        q && m(_),
        q && m(b),
        q && m(w),
        q && m(y),
        q && m(A));
    },
  };
}
function xa(l) {
  let e, t, s, n, a;
  return {
    c() {
      ((e = C('If Codon Selection is set to "codon usage", NEBaseChanger will use codon usage tables ')),
        (t = c("br")),
        (s = C(`
		  		for E. coli K12 to select the codon that has the highest usage. If multiple codons have `)),
        (n = c("br")),
        (a = C(`
		  		the same usage frequency, the choice between them is not defined and could be random.`)),
        r(t, "class", "svelte-1u6ecyq"),
        r(n, "class", "svelte-1u6ecyq"));
    },
    m(i, u) {
      (g(i, e, u), g(i, t, u), g(i, s, u), g(i, n, u), g(i, a, u));
    },
    p: qe,
    d(i) {
      (i && m(e), i && m(t), i && m(s), i && m(n), i && m(a));
    },
  };
}
function Ja(l) {
  let e, t, s, n, a, i, u;
  return {
    c() {
      ((e = C('If Codon Selection is set to "parsimony", NEBaseChanger will select the codon that has ')),
        (t = c("br")),
        (s = C(`
		  		the maximum parsimony defined here as the fewest base changes relative to the codon `)),
        (n = c("br")),
        (a = C(`
		  		being replaced. If multiple codons have the same minimum distance, the choice between them `)),
        (i = c("br")),
        (u = C(`
		  		is not defined and could be random.`)),
        r(t, "class", "svelte-1u6ecyq"),
        r(n, "class", "svelte-1u6ecyq"),
        r(i, "class", "svelte-1u6ecyq"));
    },
    m(f, h) {
      (g(f, e, h), g(f, t, h), g(f, s, h), g(f, n, h), g(f, a, h), g(f, i, h), g(f, u, h));
    },
    p: qe,
    d(f) {
      (f && m(e), f && m(t), f && m(s), f && m(n), f && m(a), f && m(i), f && m(u));
    },
  };
}
function Za(l) {
  let e, t, s, n, a, i, u, f, h, d, p, v;
  return {
    c() {
      ((e = C("By selecting this option, NEBaseChanger will generate primer pairs compatible with ")),
        (t = c("br")),
        (s = C(`
		  		the NEBuilder HiFi DNA Assembly Master Mix to simplify the execution of multiple `)),
        (n = c("br")),
        (a = C(`
		  		site-directed mutagenesis in a single reaction. `)),
        (i = c("br")),
        (u = S()),
        (f = c("br")),
        (h = C(`
		  		Details are available `)),
        (d = c("a")),
        (d.textContent = "here"),
        (p = C(".")),
        (v = c("br")),
        r(t, "class", "svelte-1u6ecyq"),
        r(n, "class", "svelte-1u6ecyq"),
        r(i, "class", "svelte-1u6ecyq"),
        r(f, "class", "svelte-1u6ecyq"),
        r(
          d,
          "href",
          "https://www.neb.com/-/media/nebus/files/application-notes/appnote_improved_methods_for_sdm_using_nebuilder_hifi_dna_assembly_master_mix.pdf?rev=918a4c52177641cbb8e2087a1286b9f4",
        ),
        r(d, "target", "_blank"),
        r(d, "class", "svelte-1u6ecyq"),
        r(v, "class", "svelte-1u6ecyq"));
    },
    m(_, b) {
      (g(_, e, b),
        g(_, t, b),
        g(_, s, b),
        g(_, n, b),
        g(_, a, b),
        g(_, i, b),
        g(_, u, b),
        g(_, f, b),
        g(_, h, b),
        g(_, d, b),
        g(_, p, b),
        g(_, v, b));
    },
    p: qe,
    d(_) {
      (_ && m(e), _ && m(t), _ && m(s), _ && m(n), _ && m(a), _ && m(i), _ && m(u), _ && m(f), _ && m(h), _ && m(d), _ && m(p), _ && m(v));
    },
  };
}
function $a(l) {
  let e, t, s, n, a, i, u, f, h, d, p, v, _, b, w, y, A;
  return {
    c() {
      ((e = C("Enter one or more mutations of the form ")),
        (t = c("br")),
        (s = C(`
		  		[old AA][codon #][new AA], for example, M2A, F32H, C45Y, etc. `)),
        (n = c("br")),
        (a = C(`
		  		Optionally, specify the exact triplet replacement by appending a colon followed by the triplet, `)),
        (i = c("br")),
        (u = C(`
		  		for example, M2G:GGG, A34F:TTC. Exact triplets, when specified, will override automatic selection.`)),
        (f = c("br")),
        (h = S()),
        (d = c("br")),
        (p = S()),
        (v = c("br")),
        (_ = C(`
		  		Allowed one letter codes:`)),
        (b = c("br")),
        (w = c("br")),
        (y = S()),
        (A = c("code")),
        (A.innerHTML = `A : Ala (Alanine)<br class="svelte-1u6ecyq"/>
		        
		        C : Cys (Cysteine)<br class="svelte-1u6ecyq"/>
		        D : Asp (Aspartic Acid)<br class="svelte-1u6ecyq"/>
		        E : Glu (Glutamic Acid)<br class="svelte-1u6ecyq"/>
		        F : Phe (Phenylalanine)<br class="svelte-1u6ecyq"/>
		        G : Gly (Glycine)<br class="svelte-1u6ecyq"/>
		        H : His (Histidine)<br class="svelte-1u6ecyq"/>
		        I : Ile (Isoleucine)<br class="svelte-1u6ecyq"/>
		        
		        K : Lys (Lysine)<br class="svelte-1u6ecyq"/>
		        L : Leu (Leucine)<br class="svelte-1u6ecyq"/>
		        M : Met (Methionine)<br class="svelte-1u6ecyq"/>
		        N : Asn (Asparagine)<br class="svelte-1u6ecyq"/>
		        
		        P : Pro (Proline)<br class="svelte-1u6ecyq"/>
		        Q : Gln (Glutamine)<br class="svelte-1u6ecyq"/>
		        R : Arg (Arginine)<br class="svelte-1u6ecyq"/>
		        S : Ser (Serine)<br class="svelte-1u6ecyq"/>
		        T : Thr (Threonine)<br class="svelte-1u6ecyq"/>
		        
		        V : Val (Valine)<br class="svelte-1u6ecyq"/>
		        W : Trp (Tryptophan)<br class="svelte-1u6ecyq"/>
		        X : Xaa (NNK - Any amino acid)<br class="svelte-1u6ecyq"/>
		        Y : Tyr (Tyrosine)<br class="svelte-1u6ecyq"/>
		        
		        * : Ter	(Termination)`),
        r(t, "class", "svelte-1u6ecyq"),
        r(n, "class", "svelte-1u6ecyq"),
        r(i, "class", "svelte-1u6ecyq"),
        r(f, "class", "svelte-1u6ecyq"),
        r(d, "class", "svelte-1u6ecyq"),
        r(v, "class", "svelte-1u6ecyq"),
        r(b, "class", "svelte-1u6ecyq"),
        r(w, "class", "svelte-1u6ecyq"),
        r(A, "class", "svelte-1u6ecyq"));
    },
    m(q, T) {
      (g(q, e, T),
        g(q, t, T),
        g(q, s, T),
        g(q, n, T),
        g(q, a, T),
        g(q, i, T),
        g(q, u, T),
        g(q, f, T),
        g(q, h, T),
        g(q, d, T),
        g(q, p, T),
        g(q, v, T),
        g(q, _, T),
        g(q, b, T),
        g(q, w, T),
        g(q, y, T),
        g(q, A, T));
    },
    p: qe,
    d(q) {
      (q && m(e),
        q && m(t),
        q && m(s),
        q && m(n),
        q && m(a),
        q && m(i),
        q && m(u),
        q && m(f),
        q && m(h),
        q && m(d),
        q && m(p),
        q && m(v),
        q && m(_),
        q && m(b),
        q && m(w),
        q && m(y),
        q && m(A));
    },
  };
}
function ec(l) {
  let e;
  return {
    c() {
      ((e = c("p")),
        (e.innerHTML = `The default minimum primer length is 15 nt. Raising this value <br class="svelte-1u6ecyq"/>
		  			will increase the specificity of the primers.  When modifications <br class="svelte-1u6ecyq"/>
		  			are constrained to the 5\u2032 tails, the minimum primer length <br class="svelte-1u6ecyq"/>
		  			only applies to the annealing portion of the primer. To match the <br class="svelte-1u6ecyq"/>
		  			specifications used in NEBaseChanger v1, set the minimum to 18 nt. <br class="svelte-1u6ecyq"/>
		  			The minimum allowed value is 15 nt.`),
        r(e, "class", "svelte-1u6ecyq"));
    },
    m(t, s) {
      g(t, e, s);
    },
    p: qe,
    d(t) {
      t && m(e);
    },
  };
}
function tc(l) {
  let e;
  return {
    c() {
      ((e = c("p")),
        (e.innerHTML = `The default minimum temperature for primer melting (Tm) is 55\xB0C.<br class="svelte-1u6ecyq"/>
		  			If higher primer binding specificity is desired or a tighter range <br class="svelte-1u6ecyq"/>
		  			of Tm values across a larger set of primers is desired, raise the <br class="svelte-1u6ecyq"/>
		  			minimum Tm value.  Primer lengths will be longer at higher minimum <br class="svelte-1u6ecyq"/>
		  			Tm values. The minimum allowed value is 55\xB0C. Maximum allowed <br class="svelte-1u6ecyq"/>
		  			value is 72\xB0C.`),
        r(e, "class", "svelte-1u6ecyq"));
    },
    m(t, s) {
      g(t, e, s);
    },
    p: qe,
    d(t) {
      t && m(e);
    },
  };
}
function lc(l) {
  let e, t, s, n, a, i, u, f, h;
  return {
    c() {
      ((e = c("p")),
        (e.textContent = "Successful primer generation output from NEBaseChanger can be viewed in multiple ways:"),
        (t = S()),
        (s = c("h4")),
        (s.textContent = "Required Primers Table"),
        (n = S()),
        (a = c("p")),
        (a.innerHTML = `Displayed near the bottom of the page. In addition to the primer sequence, <br class="svelte-1u6ecyq"/>
					the length, calculated melting temperature (Tm) and suggested <br class="svelte-1u6ecyq"/>
					annealing temperature (Ta) are displayed. The primer display is automatically <br class="svelte-1u6ecyq"/>
					updated whenever the mutation region or desired sequence is changed. Forward and <br class="svelte-1u6ecyq"/>
					reverse primers are color coded and the 3-frame coding sequence is shown <br class="svelte-1u6ecyq"/>
					allowing the user to verify that the desired mutation is incorporated.`),
        (i = S()),
        (u = c("h4")),
        (u.textContent = "Downloadable formats"),
        (f = S()),
        (h = c("dl")),
        (h.innerHTML = `<dt class="svelte-1u6ecyq">Summary table</dt><dd class="svelte-1u6ecyq">A tab-delimited table containing information comparable to the Primer Result table.
					</dd><dt class="svelte-1u6ecyq">All primers</dt><dd class="svelte-1u6ecyq">A tab-delimited plain text list of the required primers (name, sequence, note), suitable for common oligonucleotide order forms.
					</dd><dt class="svelte-1u6ecyq">Selected primers</dt><dd class="svelte-1u6ecyq">A tab-delimited plain text list of the required primers (name, sequence, note), suitable for common oligonucleotide order forms. Lited to just the selected primer pair.
					</dd><dt class="svelte-1u6ecyq">All sequence</dt><dd class="svelte-1u6ecyq">All result sequences in multi-FASTA format as a local text file.
					</dd><dt class="svelte-1u6ecyq">Selected sequence</dt><dd class="svelte-1u6ecyq">The result sequence for the selected primer pair in FASTA format as a local text file.</dd>`),
        r(e, "class", "svelte-1u6ecyq"),
        r(s, "class", "svelte-1u6ecyq"),
        r(a, "class", "svelte-1u6ecyq"),
        r(u, "class", "svelte-1u6ecyq"),
        r(h, "class", "svelte-1u6ecyq"));
    },
    m(d, p) {
      (g(d, e, p), g(d, t, p), g(d, s, p), g(d, n, p), g(d, a, p), g(d, i, p), g(d, u, p), g(d, f, p), g(d, h, p));
    },
    p: qe,
    d(d) {
      (d && m(e), d && m(t), d && m(s), d && m(n), d && m(a), d && m(i), d && m(u), d && m(f), d && m(h));
    },
  };
}
function sc(l) {
  let e,
    t = Te.cut.Ecoli_K12.slice().sort(Vr),
    s = [];
  for (let n = 0; n < t.length; n += 1) s[n] = cr(ir(l, t, n));
  return {
    c() {
      for (let n = 0; n < s.length; n += 1) s[n].c();
      e = is();
    },
    m(n, a) {
      for (let i = 0; i < s.length; i += 1) s[i] && s[i].m(n, a);
      g(n, e, a);
    },
    p(n, a) {
      if (a & 0) {
        t = Te.cut.Ecoli_K12.slice().sort(Vr);
        let i;
        for (i = 0; i < t.length; i += 1) {
          const u = ir(n, t, i);
          s[i] ? s[i].p(u, a) : ((s[i] = cr(u)), s[i].c(), s[i].m(e.parentNode, e));
        }
        for (; i < s.length; i += 1) s[i].d(1);
        s.length = t.length;
      }
    },
    d(n) {
      (cn(s, n), n && m(e));
    },
  };
}
function nc(l) {
  let e,
    t = Te.cut.Ecoli_K12,
    s = [];
  for (let n = 0; n < t.length; n += 1) s[n] = ur(rr(l, t, n));
  return {
    c() {
      for (let n = 0; n < s.length; n += 1) s[n].c();
      e = is();
    },
    m(n, a) {
      for (let i = 0; i < s.length; i += 1) s[i] && s[i].m(n, a);
      g(n, e, a);
    },
    p(n, a) {
      if (a & 0) {
        t = Te.cut.Ecoli_K12;
        let i;
        for (i = 0; i < t.length; i += 1) {
          const u = rr(n, t, i);
          s[i] ? s[i].p(u, a) : ((s[i] = ur(u)), s[i].c(), s[i].m(e.parentNode, e));
        }
        for (; i < s.length; i += 1) s[i].d(1);
        s.length = t.length;
      }
    },
    d(n) {
      (cn(s, n), n && m(e));
    },
  };
}
function cr(l) {
  let e,
    t,
    s = l[260][1] + "",
    n,
    a,
    i = l[260][0] + "",
    u;
  return {
    c() {
      ((e = c("option")),
        (t = c("span")),
        (n = C(s)),
        (a = C(" -- ")),
        (u = C(i)),
        r(t, "class", "tagname svelte-1u6ecyq"),
        (e.__value = l[260][1]),
        (e.value = e.__value),
        r(e, "class", "svelte-1u6ecyq"));
    },
    m(f, h) {
      (g(f, e, h), o(e, t), o(t, n), o(t, a), o(t, u));
    },
    p: qe,
    d(f) {
      f && m(e);
    },
  };
}
function ur(l) {
  let e,
    t,
    s = l[260][0] + "",
    n,
    a,
    i = l[260][1] + "",
    u;
  return {
    c() {
      ((e = c("option")),
        (t = c("span")),
        (n = C(s)),
        (a = C(" -- ")),
        (u = C(i)),
        r(t, "class", "tagname svelte-1u6ecyq"),
        (e.__value = l[260][1]),
        (e.value = e.__value),
        r(e, "class", "svelte-1u6ecyq"));
    },
    m(f, h) {
      (g(f, e, h), o(e, t), o(t, n), o(t, a), o(t, u));
    },
    p: qe,
    d(f) {
      f && m(e);
    },
  };
}
function fr(l) {
  let e, t, s, n;
  return {
    c() {
      ((e = c("button")),
        (t = C("Confirm")),
        r(e, "class", "button btn btn-link-cta svelte-1u6ecyq"),
        (e.value = l[6]),
        r(e, "formmethod", "dialog"));
    },
    m(a, i) {
      (g(a, e, i), o(e, t), s || ((n = ce(e, "click", l[120])), (s = !0)));
    },
    p(a, i) {
      i[0] & 64 && (e.value = a[6]);
    },
    d(a) {
      (a && m(e), (s = !1), n());
    },
  };
}
function ic(l) {
  let e,
    t,
    s = l[55].cdn + "",
    n,
    a,
    i = l[55].orfstart + "",
    u,
    f,
    h,
    d,
    p,
    v,
    _,
    b = l[55].srcaa3 + "",
    w,
    y,
    A = l[55].srccdn + "",
    q,
    T,
    N,
    M,
    k,
    E,
    P,
    G,
    O,
    K,
    D,
    L,
    R,
    Q,
    ae,
    W,
    x,
    H;
  function U(z, te) {
    return z[55].order == "cut" ? nc : sc;
  }
  let le = U(l),
    J = le(l),
    ie = l[6] != "" && fr(l);
  return {
    c() {
      ((e = c("br")),
        (t = C(`
		  		codon `)),
        (n = C(s)),
        (a = C(" in ORF starting at ")),
        (u = C(i)),
        (f = S()),
        (h = c("br")),
        (d = S()),
        (p = c("br")),
        (v = C(`
		  		Change `)),
        (_ = c("b")),
        (w = C(b)),
        (y = C(" -- ")),
        (q = C(A)),
        (T = C(`. to:
	            `)),
        (N = c("br")),
        (M = S()),
        (k = c("br")),
        (E = S()),
        (P = c("form")),
        (G = c("select")),
        (O = c("option")),
        (O.innerHTML = '<strong class="svelte-1u6ecyq">AA Codons \xA0\xA0\xA0</strong>'),
        J.c(),
        (K = S()),
        (D = c("br")),
        (L = c("br")),
        (R = S()),
        (Q = c("div")),
        ie && ie.c(),
        (ae = S()),
        (W = c("button")),
        (W.textContent = "Cancel"),
        r(e, "class", "svelte-1u6ecyq"),
        r(h, "class", "svelte-1u6ecyq"),
        r(p, "class", "svelte-1u6ecyq"),
        r(_, "class", "svelte-1u6ecyq"),
        r(N, "class", "svelte-1u6ecyq"),
        r(k, "class", "svelte-1u6ecyq"),
        (O.__value = ""),
        (O.value = O.__value),
        (O.selected = "selected"),
        r(O, "class", "svelte-1u6ecyq"),
        r(G, "id", "codontag"),
        r(G, "aria-label", "Amino acid codons"),
        r(G, "class", "dropdown neb-dropdown svelte-1u6ecyq"),
        l[6] === void 0 && vn(() => l[119].call(G)),
        r(D, "class", "svelte-1u6ecyq"),
        r(L, "class", "svelte-1u6ecyq"),
        r(W, "class", "button btn btn-link svelte-1u6ecyq"),
        (W.value = ""),
        r(W, "formmethod", "dialog"),
        r(Q, "class", "svelte-1u6ecyq"),
        r(P, "method", "dialog"),
        r(P, "class", "svelte-1u6ecyq"));
    },
    m(z, te) {
      (g(z, e, te),
        g(z, t, te),
        g(z, n, te),
        g(z, a, te),
        g(z, u, te),
        g(z, f, te),
        g(z, h, te),
        g(z, d, te),
        g(z, p, te),
        g(z, v, te),
        g(z, _, te),
        o(_, w),
        o(_, y),
        o(_, q),
        g(z, T, te),
        g(z, N, te),
        g(z, M, te),
        g(z, k, te),
        g(z, E, te),
        g(z, P, te),
        o(P, G),
        o(G, O),
        J.m(G, null),
        gn(G, l[6], !0),
        o(P, K),
        o(P, D),
        o(P, L),
        o(P, R),
        o(P, Q),
        ie && ie.m(Q, null),
        o(Q, ae),
        o(Q, W),
        x || ((H = [ce(G, "change", l[119]), ce(W, "click", l[121])]), (x = !0)));
    },
    p(z, te) {
      (te[1] & 16777216 && s !== (s = z[55].cdn + "") && Z(n, s),
        te[1] & 16777216 && i !== (i = z[55].orfstart + "") && Z(u, i),
        te[1] & 16777216 && b !== (b = z[55].srcaa3 + "") && Z(w, b),
        te[1] & 16777216 && A !== (A = z[55].srccdn + "") && Z(q, A),
        le === (le = U(z)) && J ? J.p(z, te) : (J.d(1), (J = le(z)), J && (J.c(), J.m(G, null))),
        te[0] & 64 && gn(G, z[6]),
        z[6] != "" ? (ie ? ie.p(z, te) : ((ie = fr(z)), ie.c(), ie.m(Q, ae))) : ie && (ie.d(1), (ie = null)));
    },
    d(z) {
      (z && m(e),
        z && m(t),
        z && m(n),
        z && m(a),
        z && m(u),
        z && m(f),
        z && m(h),
        z && m(d),
        z && m(p),
        z && m(v),
        z && m(_),
        z && m(T),
        z && m(N),
        z && m(M),
        z && m(k),
        z && m(E),
        z && m(P),
        J.d(),
        ie && ie.d(),
        (x = !1),
        Zt(H));
    },
  };
}
function rc(l) {
  let e;
  return {
    c() {
      ((e = c("p")),
        (e.innerHTML = `NEBuilder Calculator (https://nebuildercalculator.neb.com) can be used to generate <br class="svelte-1u6ecyq"/>
		  			a customized protocol for the NEBuilder HiFi DNA Assembly Master Mix with calculated <br class="svelte-1u6ecyq"/>
		  			optimal volumes of input DNA fragment sequences of each fragment.<br class="svelte-1u6ecyq"/>
		  			The primers generated with the NEBuilder HiFi option selected are used to calculate <br class="svelte-1u6ecyq"/>
		  			the expected PCR fragment lengths. This data is sent to NEBuilder Calculator <br class="svelte-1u6ecyq"/>
		  			and opened in a new browser tab.`),
        r(e, "class", "svelte-1u6ecyq"));
    },
    m(t, s) {
      g(t, e, s);
    },
    p: qe,
    d(t) {
      t && m(e);
    },
  };
}
function oc(l) {
  let e, t, s, n, a, i, u, f, h, d, p, v, _, b, w, y, A, q, T, N, M;
  ((u = new Rl({ props: { topic: "Allowed Batch Input" } })), u.$on("click", l[170]));
  let k = l[62].length > 0 && dr(l);
  return {
    c() {
      ((e = c("hr")),
        (t = S()),
        (s = c("div")),
        (n = c("div")),
        (a = c("span")),
        (a.textContent = "Type/paste operations below or load from file:"),
        (i = S()),
        Re(u.$$.fragment),
        (f = S()),
        (h = c("form")),
        (d = c("input")),
        (p = S()),
        (v = c("br")),
        (_ = S()),
        (b = c("textarea")),
        (w = S()),
        (y = c("div")),
        k && k.c(),
        (A = S()),
        (q = c("hr")),
        r(e, "class", "svelte-1u6ecyq"),
        r(a, "class", "svelte-1u6ecyq"),
        r(d, "id", "batchfileinput"),
        r(d, "type", "file"),
        r(d, "aria-label", "Load a batch file"),
        r(d, "class", "svelte-1u6ecyq"),
        r(h, "id", "batchfileform"),
        r(h, "name", "batchfileform"),
        r(h, "class", "svelte-1u6ecyq"),
        r(n, "id", "fileinputcontainer"),
        r(n, "class", "svelte-1u6ecyq"),
        r(v, "class", "svelte-1u6ecyq"),
        r(b, "id", "batchinputops"),
        r(b, "type", "text"),
        r(b, "class", "longseqinput svelte-1u6ecyq"),
        r(b, "aria-label", "Batch mutation commands"),
        r(b, "placeholder", "Check Help for command specifications and formatting"),
        r(y, "class", "svelte-1u6ecyq"),
        r(s, "id", "batchinput"),
        r(s, "class", "svelte-1u6ecyq"),
        r(q, "class", "svelte-1u6ecyq"));
    },
    m(E, P) {
      (g(E, e, P),
        g(E, t, P),
        g(E, s, P),
        o(s, n),
        o(n, a),
        o(n, i),
        Ge(u, n, null),
        o(n, f),
        o(n, h),
        o(h, d),
        l[172](h),
        o(s, p),
        o(s, v),
        o(s, _),
        o(s, b),
        nt(b, l[22]),
        o(s, w),
        o(s, y),
        k && k.m(y, null),
        g(E, A, P),
        g(E, q, P),
        (T = !0),
        N || ((M = [ce(d, "change", l[171]), ce(d, "change", l[66]), ce(b, "input", l[173]), ce(b, "blur", l[77])]), (N = !0)));
    },
    p(E, P) {
      (P[0] & 4194304 && nt(b, E[22]), E[62].length > 0 ? (k ? k.p(E, P) : ((k = dr(E)), k.c(), k.m(y, null))) : k && (k.d(1), (k = null)));
    },
    i(E) {
      T || (ve(u.$$.fragment, E), (T = !0));
    },
    o(E) {
      (be(u.$$.fragment, E), (T = !1));
    },
    d(E) {
      (E && m(e), E && m(t), E && m(s), Fe(u), l[172](null), k && k.d(), E && m(A), E && m(q), (N = !1), Zt(M));
    },
  };
}
function ac(l) {
  let e,
    t,
    s,
    n,
    a,
    i,
    u,
    f,
    h,
    d,
    p,
    v,
    _,
    b,
    w,
    y,
    A,
    q,
    T,
    N,
    M,
    k,
    E,
    P,
    G,
    O,
    K,
    D,
    L,
    R,
    Q,
    ae,
    W,
    x,
    H,
    U,
    le,
    J,
    ie,
    z,
    te,
    pe,
    Y,
    ue,
    he,
    de,
    _e,
    ne,
    $,
    fe,
    we,
    j,
    F,
    ee,
    V,
    me,
    _t,
    Je,
    bt;
  const ut = [dc, fc],
    it = [];
  function Be(Ce, De) {
    return Ce[0] && Ce[0].length > 0 ? 0 : 1;
  }
  ((A = Be(l)),
    (q = it[A] = ut[A](l)),
    (K = new Rl({ props: { topic: "Codon selection: codon usage" } })),
    K.$on("click", l[162]),
    (x = new Rl({ props: { topic: "Codon selection: parsimony" } })),
    x.$on("click", l[164]),
    (te = new Rl({ props: { topic: "Mutation One-Letter-Codes" } })),
    te.$on("click", l[165]));
  let We = l[62].length > 0 && hr(l),
    Ie = !l[17] && gr(l);
  return (
    (_t = Ti(l[127][0])),
    {
      c() {
        ((e = c("hr")),
          (t = S()),
          (s = c("div")),
          (n = c("div")),
          (a = c("label")),
          (a.textContent = "ORF start position:"),
          (i = S()),
          (u = c("input")),
          (f = S()),
          (h = c("button")),
          (h.innerHTML = '<strong class="svelte-1u6ecyq">reset to 1</strong>'),
          (d = S()),
          (p = c("button")),
          (p.innerHTML = '<strong class="svelte-1u6ecyq">&lt;&lt; prev</strong>'),
          (v = S()),
          (_ = c("button")),
          (_.innerHTML = '<strong class="svelte-1u6ecyq">next &gt;&gt;</strong>'),
          (b = S()),
          (w = c("br")),
          (y = S()),
          q.c(),
          (T = S()),
          (N = c("div")),
          (M = C(`Codon selection: \xA0\xA0
	      				`)),
          (k = c("span")),
          (E = c("input")),
          (P = S()),
          (G = c("label")),
          (G.textContent = "codon usage"),
          (O = S()),
          Re(K.$$.fragment),
          (D = S()),
          (L = c("span")),
          (R = c("input")),
          (Q = S()),
          (ae = c("label")),
          (ae.textContent = "parsimony"),
          (W = S()),
          Re(x.$$.fragment),
          (H = S()),
          (U = c("br")),
          (le = S()),
          (J = c("div")),
          (ie = c("label")),
          (z = C("Enter mutations: ")),
          Re(te.$$.fragment),
          (pe = c("br")),
          (Y = S()),
          (ue = c("input")),
          (he = S()),
          (de = c("button")),
          (de.innerHTML = '<strong class="svelte-1u6ecyq">clear X</strong>'),
          (_e = S()),
          (ne = c("br")),
          ($ = S()),
          (fe = c("div")),
          (fe.textContent = "\xA0"),
          (we = S()),
          (j = c("div")),
          We && We.c(),
          (F = S()),
          Ie && Ie.c(),
          (ee = S()),
          (V = c("hr")),
          r(e, "class", "svelte-1u6ecyq"),
          r(a, "for", "offset"),
          r(a, "class", "svelte-1u6ecyq"),
          r(u, "type", "number"),
          r(u, "id", "offset"),
          r(u, "min", "1"),
          r(u, "class", "svelte-1u6ecyq"),
          r(h, "class", "button btn btn-link svelte-1u6ecyq"),
          r(p, "class", "button btn btn-link svelte-1u6ecyq"),
          r(_, "class", "button btn btn-link svelte-1u6ecyq"),
          r(n, "id", "ORFoffset"),
          r(n, "class", "range-holder svelte-1u6ecyq"),
          r(w, "class", "svelte-1u6ecyq"),
          r(E, "type", "radio"),
          r(E, "id", "maxcut"),
          r(E, "name", "cdnsort"),
          (E.__value = "maxcut"),
          (E.value = E.__value),
          r(E, "class", "svelte-1u6ecyq"),
          r(G, "for", "maxcut"),
          r(G, "class", "svelte-1u6ecyq"),
          r(k, "class", "svelte-1u6ecyq"),
          r(R, "type", "radio"),
          r(R, "id", "mindist"),
          r(R, "name", "cdnsort"),
          (R.__value = "mindist"),
          (R.value = R.__value),
          r(R, "class", "svelte-1u6ecyq"),
          r(ae, "for", "mindist"),
          r(ae, "class", "svelte-1u6ecyq"),
          r(L, "class", "svelte-1u6ecyq"),
          r(N, "class", " svelte-1u6ecyq"),
          r(U, "class", "svelte-1u6ecyq"),
          r(pe, "class", "svelte-1u6ecyq"),
          r(ue, "type", "text"),
          r(ue, "id", "codonops"),
          r(ue, "class", "longinput svelte-1u6ecyq"),
          r(ue, "placeholder", "L2R G31H T37Y"),
          r(de, "class", "button btn btn-link svelte-1u6ecyq"),
          r(ie, "for", "codonops"),
          r(ie, "class", "svelte-1u6ecyq"),
          r(ne, "class", "svelte-1u6ecyq"),
          r(fe, "id", "codonnotesindicator"),
          r(fe, "class", "svelte-1u6ecyq"),
          r(J, "id", "codonchanges"),
          r(J, "class", "svelte-1u6ecyq"),
          r(s, "id", "singleinput_codon"),
          r(s, "class", "svelte-1u6ecyq"),
          r(j, "class", "svelte-1u6ecyq"),
          r(V, "class", "svelte-1u6ecyq"),
          _t.p(E, R));
      },
      m(Ce, De) {
        (g(Ce, e, De),
          g(Ce, t, De),
          g(Ce, s, De),
          o(s, n),
          o(n, a),
          o(n, i),
          o(n, u),
          nt(u, l[14]),
          o(n, f),
          o(n, h),
          o(n, d),
          o(n, p),
          o(n, v),
          o(n, _),
          o(s, b),
          o(s, w),
          o(s, y),
          it[A].m(s, null),
          o(s, T),
          o(s, N),
          o(N, M),
          o(N, k),
          o(k, E),
          (E.checked = E.__value === l[15]),
          o(k, P),
          o(k, G),
          o(N, O),
          Ge(K, N, null),
          o(N, D),
          o(N, L),
          o(L, R),
          (R.checked = R.__value === l[15]),
          o(L, Q),
          o(L, ae),
          o(N, W),
          Ge(x, N, null),
          o(s, H),
          o(s, U),
          o(s, le),
          o(s, J),
          o(J, ie),
          o(ie, z),
          Ge(te, ie, null),
          o(ie, pe),
          o(ie, Y),
          o(ie, ue),
          nt(ue, l[13]),
          o(ie, he),
          o(ie, de),
          o(J, _e),
          o(J, ne),
          o(J, $),
          o(J, fe),
          g(Ce, we, De),
          g(Ce, j, De),
          We && We.m(j, null),
          g(Ce, F, De),
          Ie && Ie.m(Ce, De),
          g(Ce, ee, De),
          g(Ce, V, De),
          (me = !0),
          Je ||
            ((bt = [
              ce(u, "input", l[160]),
              ce(h, "click", l[93]),
              ce(p, "click", l[87]),
              ce(_, "click", l[88]),
              ce(E, "change", l[161]),
              ce(R, "change", l[163]),
              ce(ue, "input", l[166]),
              ce(ue, "keydown", l[167]),
              ce(de, "click", l[92]),
            ]),
            (Je = !0)));
      },
      p(Ce, De) {
        De[0] & 16384 && Us(u.value) !== Ce[14] && nt(u, Ce[14]);
        let at = A;
        ((A = Be(Ce)),
          A === at
            ? it[A].p(Ce, De)
            : (vs(),
              be(it[at], 1, 1, () => {
                it[at] = null;
              }),
              _s(),
              (q = it[A]),
              q ? q.p(Ce, De) : ((q = it[A] = ut[A](Ce)), q.c()),
              ve(q, 1),
              q.m(s, T)),
          De[0] & 32768 && (E.checked = E.__value === Ce[15]),
          De[0] & 32768 && (R.checked = R.__value === Ce[15]),
          De[0] & 8192 && ue.value !== Ce[13] && nt(ue, Ce[13]),
          Ce[62].length > 0 ? (We ? We.p(Ce, De) : ((We = hr(Ce)), We.c(), We.m(j, null))) : We && (We.d(1), (We = null)),
          Ce[17]
            ? Ie &&
              (vs(),
              be(Ie, 1, 1, () => {
                Ie = null;
              }),
              _s())
            : Ie
              ? (Ie.p(Ce, De), De[0] & 131072 && ve(Ie, 1))
              : ((Ie = gr(Ce)), Ie.c(), ve(Ie, 1), Ie.m(ee.parentNode, ee)));
      },
      i(Ce) {
        me || (ve(q), ve(K.$$.fragment, Ce), ve(x.$$.fragment, Ce), ve(te.$$.fragment, Ce), ve(Ie), (me = !0));
      },
      o(Ce) {
        (be(q), be(K.$$.fragment, Ce), be(x.$$.fragment, Ce), be(te.$$.fragment, Ce), be(Ie), (me = !1));
      },
      d(Ce) {
        (Ce && m(e),
          Ce && m(t),
          Ce && m(s),
          it[A].d(),
          Fe(K),
          Fe(x),
          Fe(te),
          Ce && m(we),
          Ce && m(j),
          We && We.d(),
          Ce && m(F),
          Ie && Ie.d(Ce),
          Ce && m(ee),
          Ce && m(V),
          _t.r(),
          (Je = !1),
          Zt(bt));
      },
    }
  );
}
function cc(l) {
  let e,
    t,
    s,
    n = navigator.userAgent.includes("Safari") && !navigator.userAgent.includes("Chrome") && l[0].length > 1e4,
    a,
    i,
    u,
    f,
    h,
    d,
    p,
    v,
    _,
    b,
    w,
    y,
    A,
    q,
    T,
    N,
    M,
    k,
    E,
    P,
    G,
    O,
    K,
    D,
    L,
    R,
    Q,
    ae,
    W,
    x,
    H,
    U,
    le,
    J,
    ie,
    z,
    te,
    pe,
    Y,
    ue,
    he,
    de,
    _e,
    ne,
    $,
    fe,
    we,
    j,
    F,
    ee,
    V,
    me,
    _t,
    Je,
    bt,
    ut,
    it,
    Be,
    We,
    Ie,
    Ce,
    De,
    at,
    Ve,
    $t,
    rt,
    Xt,
    Xe,
    Oe = n && vr(),
    He = l[58].length > 0 && _r(l),
    Ke = l[58].length > 0 && br(l);
  const ke = [vc, gc],
    Ue = [];
  function nl(Ee, Qe) {
    return Ee[0] && Ee[0].length > 0 ? 0 : 1;
  }
  ((H = nl(l)), (U = Ue[H] = ke[H](l)));
  let At = l[62].length > 0 && qr(l),
    Gt = Te.cut.Ecoli_K12,
    pt = [];
  for (let Ee = 0; Ee < Gt.length; Ee += 1) pt[Ee] = yr(lr(l, Gt, Ee));
  return {
    c() {
      ((e = c("hr")),
        (t = S()),
        (s = c("div")),
        Oe && Oe.c(),
        (a = S()),
        (i = c("div")),
        (u = c("span")),
        (u.textContent = "[optional] Search for target sequence:"),
        (f = C(`\xA0\xA0
	            	`)),
        He && He.c(),
        (h = S()),
        (d = c("br")),
        (p = S()),
        (v = c("input")),
        (_ = S()),
        (b = c("button")),
        (b.innerHTML = '<strong class="svelte-1u6ecyq">find &gt;</strong>'),
        (w = S()),
        (y = c("button")),
        (y.innerHTML = '<strong class="svelte-1u6ecyq">clear X</strong>'),
        (A = S()),
        Ke && Ke.c(),
        (q = S()),
        (T = c("br")),
        (N = S()),
        (M = c("div")),
        (M.innerHTML = 'Enter the coordinates of the bases <u class="svelte-1u6ecyq">flanking</u> the targeted mutation site.'),
        (k = S()),
        (E = c("div")),
        (P = c("label")),
        (P.innerHTML = '5\u2032 <em class="svelte-1u6ecyq"><u class="svelte-1u6ecyq">flank coord.</u></em> (start)'),
        (G = S()),
        (O = c("input")),
        (K = C(`
	                \xA0 \xA0
	                `)),
        (D = c("label")),
        (D.innerHTML = '3\u2032 <em class="svelte-1u6ecyq"><u class="svelte-1u6ecyq">flank coord.</u></em> (end)'),
        (L = S()),
        (R = c("input")),
        (Q = C(`
	                \xA0 \xA0`)),
        (ae = S()),
        (W = c("br")),
        (x = S()),
        U.c(),
        (le = S()),
        (J = c("div")),
        At && At.c(),
        (ie = S()),
        (z = c("br")),
        (te = S()),
        (pe = c("div")),
        (Y = c("label")),
        (ue = C("Replacement Sequence (ambiguity allowed) -- leave blank if deleting target.")),
        (he = c("br")),
        (de = S()),
        (_e = c("input")),
        (ne = C(`
	                \xA0\xA0
	                `)),
        ($ = c("select")),
        (fe = c("option")),
        (fe.innerHTML = '<strong class="svelte-1u6ecyq">Common Peptide Tags</strong>'),
        (we = c("option")),
        (we.innerHTML = `<span class="tagname svelte-1u6ecyq">6x His</span>
	                        --
	                        <span class="tagaa svelte-1u6ecyq">HHHHHH</span> 
	                    `),
        (j = c("option")),
        (j.innerHTML = `<span class="tagname svelte-1u6ecyq">MYC</span>
	                        --
	                        <span class="tagaa svelte-1u6ecyq">EQKLISEEDL</span> 
	                    `),
        (F = c("option")),
        (F.innerHTML = `<span class="tagname svelte-1u6ecyq">FLAG</span>
	                        --
	                        <span class="tagaa svelte-1u6ecyq">DYKDDDDK</span> 
	                    `),
        (ee = c("option")),
        (ee.innerHTML = `<span class="tagname svelte-1u6ecyq">HA</span>
	                        --
	                        <span class="tagaa svelte-1u6ecyq">YPYDVPDYA</span> 
	                    `),
        (V = c("option")),
        (V.innerHTML = `<span class="tagname svelte-1u6ecyq">Strep</span>
	                        --
	                        <span class="tagaa svelte-1u6ecyq">WSHPQFEK</span> 
	                    `),
        (me = c("option")),
        (me.innerHTML = `<span class="tagname svelte-1u6ecyq">Xpress</span>
	                        --
	                        <span class="tagaa svelte-1u6ecyq">DLYDDDDK</span> 
	                    `),
        (_t = c("option")),
        (_t.innerHTML = `<span class="tagname svelte-1u6ecyq">AviTag</span>
	                        --
	                        <span class="tagaa svelte-1u6ecyq">GLNDIFEAQKIEWH</span> 
	                    `),
        (Je = c("option")),
        (Je.innerHTML = `<span class="tagname svelte-1u6ecyq">V5</span>
	                        --
	                        <span class="tagaa svelte-1u6ecyq">GKPIPNPLLGLDST</span> 
	                    `),
        (bt = c("option")),
        (bt.innerHTML = `<span class="tagname svelte-1u6ecyq">NLS</span>
	                        --
	                        <span class="tagaa svelte-1u6ecyq">PKKKRKV</span> 
	                    `),
        (ut = c("option")),
        (ut.innerHTML = `<span class="tagname svelte-1u6ecyq">SPOT</span>
	                        --
	                        <span class="tagaa svelte-1u6ecyq">PDRVRAVSHWSS</span>`),
        (it = S()),
        (Be = c("select")),
        (We = c("option")),
        (We.innerHTML = '<strong class="svelte-1u6ecyq">AA -- Codon \xA0\xA0\xA0</strong>'));
      for (let Ee = 0; Ee < pt.length; Ee += 1) pt[Ee].c();
      ((Ie = C(`
	                \xA0\xA0
	                `)),
        (Ce = c("button")),
        (Ce.innerHTML = '<strong class="svelte-1u6ecyq">clear X</strong>'),
        (De = S()),
        (at = c("div")),
        (at.textContent = "\xA0"),
        (Ve = S()),
        ($t = c("hr")),
        r(e, "class", "svelte-1u6ecyq"),
        r(u, "class", "svelte-1u6ecyq"),
        r(d, "class", "svelte-1u6ecyq"),
        r(v, "type", "text"),
        r(v, "class", "targetinput svelte-1u6ecyq"),
        r(v, "aria-label", "Search for a target sequence"),
        r(b, "class", "button btn btn-link svelte-1u6ecyq"),
        r(y, "class", "button btn btn-link svelte-1u6ecyq"),
        r(i, "id", "searchbox"),
        r(i, "class", "svelte-1u6ecyq"),
        r(T, "class", "svelte-1u6ecyq"),
        r(M, "class", "svelte-1u6ecyq"),
        r(P, "for", "selStart"),
        r(P, "class", "svelte-1u6ecyq"),
        r(O, "type", "number"),
        r(O, "min", "1"),
        r(O, "id", "selStart"),
        r(O, "class", "svelte-1u6ecyq"),
        r(D, "for", "selEnd"),
        r(D, "class", "svelte-1u6ecyq"),
        r(R, "type", "number"),
        r(R, "min", "1"),
        r(R, "id", "selEnd"),
        r(R, "class", "svelte-1u6ecyq"),
        r(E, "id", "range"),
        r(E, "class", "range-holder svelte-1u6ecyq"),
        r(W, "class", "svelte-1u6ecyq"),
        r(J, "class", "svelte-1u6ecyq"),
        r(z, "class", "svelte-1u6ecyq"),
        r(he, "class", "svelte-1u6ecyq"),
        r(_e, "type", "text"),
        r(_e, "id", "repseq"),
        r(_e, "class", "longinput svelte-1u6ecyq"),
        r(Y, "for", "repseq"),
        r(Y, "class", "svelte-1u6ecyq"),
        (fe.__value = ""),
        (fe.value = fe.__value),
        (fe.selected = "selected"),
        r(fe, "class", "svelte-1u6ecyq"),
        (we.__value = "CAT CAT CAT CAC CAC CAC"),
        (we.value = we.__value),
        r(we, "class", "svelte-1u6ecyq"),
        (j.__value = "GAA CAG AAA CTG ATT AGC GAA GAA GAT CTG"),
        (j.value = j.__value),
        r(j, "class", "svelte-1u6ecyq"),
        (F.__value = "GAT TAT AAA GAT GAT GAT GAT AAA"),
        (F.value = F.__value),
        r(F, "class", "svelte-1u6ecyq"),
        (ee.__value = "TAT CCG TAT GAT GTG CCG GAT TAT GCG"),
        (ee.value = ee.__value),
        r(ee, "class", "svelte-1u6ecyq"),
        (V.__value = "TGG AGC CAT CCG CAG TTT GAA AAA"),
        (V.value = V.__value),
        r(V, "class", "svelte-1u6ecyq"),
        (me.__value = "GAT CTG TAT GAT GAT GAT GAT AAA"),
        (me.value = me.__value),
        r(me, "class", "svelte-1u6ecyq"),
        (_t.__value = "GGC CTG AAC GAT ATT TTT GAA GCG CAG AAA ATT GAA TGG CAT GAA"),
        (_t.value = _t.__value),
        r(_t, "class", "svelte-1u6ecyq"),
        (Je.__value = "GGC AAA CCG ATT CCG AAC CCG CTG CTG GGC CTG GAT AGC ACC"),
        (Je.value = Je.__value),
        r(Je, "class", "svelte-1u6ecyq"),
        (bt.__value = "CCG AAA AAA AAA CGC AAA GTG"),
        (bt.value = bt.__value),
        r(bt, "class", "svelte-1u6ecyq"),
        (ut.__value = " CCG GAC CGT GTC CGT GCA GTC TCT CAC TGG TCC TCG "),
        (ut.value = ut.__value),
        r(ut, "class", "svelte-1u6ecyq"),
        r($, "id", "peptag"),
        r($, "aria-label", "Common peptide tags"),
        r($, "class", "dropdown neb-dropdown svelte-1u6ecyq"),
        l[5] === void 0 && vn(() => l[158].call($)),
        (We.__value = ""),
        (We.value = We.__value),
        (We.selected = "selected"),
        r(We, "class", "svelte-1u6ecyq"),
        r(Be, "id", "codontag"),
        r(Be, "aria-label", "Amino acid codons"),
        r(Be, "class", "dropdown neb-dropdown svelte-1u6ecyq"),
        l[6] === void 0 && vn(() => l[159].call(Be)),
        r(Ce, "class", "button btn btn-link svelte-1u6ecyq"),
        r(at, "id", "notesindicator"),
        r(at, "class", "svelte-1u6ecyq"),
        r(pe, "id", "replacement"),
        r(pe, "class", "svelte-1u6ecyq"),
        r(s, "id", "singleinput_general"),
        r(s, "class", "svelte-1u6ecyq"),
        r($t, "class", "svelte-1u6ecyq"));
    },
    m(Ee, Qe) {
      (g(Ee, e, Qe),
        g(Ee, t, Qe),
        g(Ee, s, Qe),
        Oe && Oe.m(s, null),
        o(s, a),
        o(s, i),
        o(i, u),
        o(i, f),
        He && He.m(i, null),
        o(i, h),
        o(i, d),
        o(i, p),
        o(i, v),
        nt(v, l[23]),
        o(i, _),
        o(i, b),
        o(i, w),
        o(i, y),
        o(i, A),
        Ke && Ke.m(i, null),
        o(s, q),
        o(s, T),
        o(s, N),
        o(s, M),
        o(s, k),
        o(s, E),
        o(E, P),
        o(E, G),
        o(E, O),
        nt(O, l[2]),
        o(E, K),
        o(E, D),
        o(E, L),
        o(E, R),
        nt(R, l[3]),
        o(E, Q),
        o(s, ae),
        o(s, W),
        o(s, x),
        Ue[H].m(s, null),
        o(s, le),
        o(s, J),
        At && At.m(J, null),
        o(s, ie),
        o(s, z),
        o(s, te),
        o(s, pe),
        o(pe, Y),
        o(Y, ue),
        o(Y, he),
        o(Y, de),
        o(Y, _e),
        nt(_e, l[4]),
        o(pe, ne),
        o(pe, $),
        o($, fe),
        o($, we),
        o($, j),
        o($, F),
        o($, ee),
        o($, V),
        o($, me),
        o($, _t),
        o($, Je),
        o($, bt),
        o($, ut),
        gn($, l[5], !0),
        o(pe, it),
        o(pe, Be),
        o(Be, We));
      for (let ht = 0; ht < pt.length; ht += 1) pt[ht] && pt[ht].m(Be, null);
      (gn(Be, l[6], !0),
        o(pe, Ie),
        o(pe, Ce),
        o(pe, De),
        o(pe, at),
        g(Ee, Ve, Qe),
        g(Ee, $t, Qe),
        (rt = !0),
        Xt ||
          ((Xe = [
            ce(v, "input", l[152]),
            ce(v, "change", l[153]),
            ce(b, "click", l[81]),
            ce(y, "click", l[89]),
            ce(O, "input", l[154]),
            ce(R, "input", l[155]),
            ce(_e, "input", l[157]),
            ce(_e, "blur", l[80]),
            ce($, "change", l[158]),
            ce(Be, "change", l[159]),
            ce(Ce, "click", l[90]),
          ]),
          (Xt = !0)));
    },
    p(Ee, Qe) {
      (Qe[0] & 1 && (n = navigator.userAgent.includes("Safari") && !navigator.userAgent.includes("Chrome") && Ee[0].length > 1e4),
        n ? Oe || ((Oe = vr()), Oe.c(), Oe.m(s, a)) : Oe && (Oe.d(1), (Oe = null)),
        Ee[58].length > 0 ? (He ? He.p(Ee, Qe) : ((He = _r(Ee)), He.c(), He.m(i, h))) : He && (He.d(1), (He = null)),
        Qe[0] & 8388608 && v.value !== Ee[23] && nt(v, Ee[23]),
        Ee[58].length > 0 ? (Ke ? Ke.p(Ee, Qe) : ((Ke = br(Ee)), Ke.c(), Ke.m(i, null))) : Ke && (Ke.d(1), (Ke = null)),
        Qe[0] & 4 && Us(O.value) !== Ee[2] && nt(O, Ee[2]),
        Qe[0] & 8 && Us(R.value) !== Ee[3] && nt(R, Ee[3]));
      let ht = H;
      if (
        ((H = nl(Ee)),
        H === ht
          ? Ue[H].p(Ee, Qe)
          : (vs(),
            be(Ue[ht], 1, 1, () => {
              Ue[ht] = null;
            }),
            _s(),
            (U = Ue[H]),
            U ? U.p(Ee, Qe) : ((U = Ue[H] = ke[H](Ee)), U.c()),
            ve(U, 1),
            U.m(s, le)),
        Ee[62].length > 0 ? (At ? At.p(Ee, Qe) : ((At = qr(Ee)), At.c(), At.m(J, null))) : At && (At.d(1), (At = null)),
        Qe[0] & 16 && _e.value !== Ee[4] && nt(_e, Ee[4]),
        Qe[0] & 32 && gn($, Ee[5]),
        Qe & 0)
      ) {
        Gt = Te.cut.Ecoli_K12;
        let tt;
        for (tt = 0; tt < Gt.length; tt += 1) {
          const pl = lr(Ee, Gt, tt);
          pt[tt] ? pt[tt].p(pl, Qe) : ((pt[tt] = yr(pl)), pt[tt].c(), pt[tt].m(Be, null));
        }
        for (; tt < pt.length; tt += 1) pt[tt].d(1);
        pt.length = Gt.length;
      }
      Qe[0] & 64 && gn(Be, Ee[6]);
    },
    i(Ee) {
      rt || (ve(U), (rt = !0));
    },
    o(Ee) {
      (be(U), (rt = !1));
    },
    d(Ee) {
      (Ee && m(e),
        Ee && m(t),
        Ee && m(s),
        Oe && Oe.d(),
        He && He.d(),
        Ke && Ke.d(),
        Ue[H].d(),
        At && At.d(),
        cn(pt, Ee),
        Ee && m(Ve),
        Ee && m($t),
        (Xt = !1),
        Zt(Xe));
    },
  };
}
function uc(l) {
  let e,
    t,
    s,
    n,
    a,
    i,
    u,
    f,
    h,
    d,
    p,
    v,
    _,
    b,
    w,
    y,
    A,
    q,
    T,
    N,
    M,
    k,
    E,
    P,
    G,
    O,
    K,
    D,
    L,
    R,
    Q,
    ae,
    W,
    x,
    H,
    U,
    le,
    J,
    ie,
    z,
    te,
    pe,
    Y = l[64][l[24]] + "",
    ue,
    he,
    de,
    _e,
    ne,
    $,
    fe,
    we,
    j,
    F,
    ee,
    V,
    me,
    _t,
    Je,
    bt,
    ut,
    it,
    Be,
    We,
    Ie,
    Ce,
    De,
    at,
    Ve;
  const $t = [bc, _c],
    rt = [];
  function Xt(ke, Ue) {
    return ke[0] && ke[0].length > 0 ? 0 : 1;
  }
  ((a = Xt(l)), (i = rt[a] = $t[a](l)));
  let Xe = l[58].length > 0 && Cr(l),
    Oe = l[58].length > 0 && Tr(l),
    He = l[62].length > 0 && Sr(l),
    Ke = l[24] != "Deletion" && Mr(l);
  return {
    c() {
      ((e = c("hr")),
        (t = S()),
        (s = c("div")),
        (n = c("div")),
        i.c(),
        (u = S()),
        (f = c("div")),
        (h = c("div")),
        (d = c("button")),
        (p = C("Substitution")),
        (_ = S()),
        (b = c("button")),
        (w = C("Insertion")),
        (A = S()),
        (q = c("button")),
        (T = C("Deletion")),
        (M = S()),
        (k = c("br")),
        (E = S()),
        (P = c("div")),
        (G = c("span")),
        (G.textContent = "[optional] Search for target sequence:"),
        (O = C(`\xA0\xA0
		            	`)),
        Xe && Xe.c(),
        (K = S()),
        (D = c("br")),
        (L = S()),
        (R = c("input")),
        (Q = S()),
        (ae = c("button")),
        (ae.innerHTML = '<strong class="svelte-1u6ecyq">find &gt;</strong>'),
        (W = S()),
        (x = c("button")),
        (x.innerHTML = '<strong class="svelte-1u6ecyq">clear X</strong>'),
        (H = S()),
        (U = c("br")),
        (le = S()),
        Oe && Oe.c(),
        (J = S()),
        (ie = c("br")),
        (z = c("br")),
        (te = S()),
        (pe = c("div")),
        (ue = C(Y)),
        (he = S()),
        (de = c("div")),
        (_e = c("label")),
        (_e.innerHTML =
          '5\u2032 <em class="svelte-1u6ecyq"><u class="svelte-1u6ecyq"><b class="svelte-1u6ecyq">flank</b> coord.</u></em> (start)'),
        (ne = S()),
        ($ = c("input")),
        (fe = C(`
		                \xA0 \xA0
		                `)),
        (we = c("label")),
        (we.innerHTML =
          '3\u2032 <em class="svelte-1u6ecyq"><u class="svelte-1u6ecyq"><b class="svelte-1u6ecyq">flank</b> coord.</u></em> (end)'),
        (j = S()),
        (F = c("input")),
        (ee = C(`
		                \xA0 \xA0`)),
        (V = S()),
        (me = c("br")),
        (_t = S()),
        (Je = c("br")),
        (bt = S()),
        (ut = c("div")),
        He && He.c(),
        (it = S()),
        (Be = c("br")),
        (We = S()),
        Ke && Ke.c(),
        (Ie = S()),
        (Ce = c("hr")),
        r(e, "class", "svelte-1u6ecyq"),
        r(n, "class", "column-left svelte-1u6ecyq"),
        r(d, "class", (v = Vs(l[24] === "Substitution" ? "active" : "") + " svelte-1u6ecyq")),
        r(b, "class", (y = Vs(l[24] === "Insertion" ? "active" : "") + " svelte-1u6ecyq")),
        r(q, "class", (N = Vs(l[24] === "Deletion" ? "active" : "") + " svelte-1u6ecyq")),
        r(h, "class", "button-bar svelte-1u6ecyq"),
        r(k, "class", "svelte-1u6ecyq"),
        r(G, "class", "svelte-1u6ecyq"),
        r(D, "class", "svelte-1u6ecyq"),
        r(R, "type", "text"),
        r(R, "class", "targetinput svelte-1u6ecyq"),
        r(R, "aria-label", "Search for a target sequence"),
        r(ae, "class", "button btn btn-link svelte-1u6ecyq"),
        r(x, "class", "button btn btn-link svelte-1u6ecyq"),
        r(U, "class", "svelte-1u6ecyq"),
        r(P, "id", "searchbox"),
        r(P, "class", "svelte-1u6ecyq"),
        r(ie, "class", "svelte-1u6ecyq"),
        r(z, "class", "svelte-1u6ecyq"),
        r(pe, "class", "svelte-1u6ecyq"),
        r(_e, "for", "selStart"),
        r(_e, "class", "svelte-1u6ecyq"),
        r($, "type", "number"),
        r($, "min", "1"),
        r($, "id", "selStart"),
        r($, "class", "svelte-1u6ecyq"),
        r(we, "for", "selEnd"),
        r(we, "class", "svelte-1u6ecyq"),
        r(F, "type", "number"),
        r(F, "min", "1"),
        r(F, "id", "selEnd"),
        r(F, "class", "svelte-1u6ecyq"),
        r(de, "id", "range"),
        r(de, "class", "range-holder svelte-1u6ecyq"),
        r(me, "class", "svelte-1u6ecyq"),
        r(Je, "class", "svelte-1u6ecyq"),
        r(ut, "class", "svelte-1u6ecyq"),
        r(Be, "class", "svelte-1u6ecyq"),
        r(f, "class", "column-right svelte-1u6ecyq"),
        r(s, "id", "singleinput_legacy"),
        r(s, "class", "flex-container svelte-1u6ecyq"),
        r(Ce, "class", "svelte-1u6ecyq"));
    },
    m(ke, Ue) {
      (g(ke, e, Ue),
        g(ke, t, Ue),
        g(ke, s, Ue),
        o(s, n),
        rt[a].m(n, null),
        o(s, u),
        o(s, f),
        o(f, h),
        o(h, d),
        o(d, p),
        o(h, _),
        o(h, b),
        o(b, w),
        o(h, A),
        o(h, q),
        o(q, T),
        o(f, M),
        o(f, k),
        o(f, E),
        o(f, P),
        o(P, G),
        o(P, O),
        Xe && Xe.m(P, null),
        o(P, K),
        o(P, D),
        o(P, L),
        o(P, R),
        nt(R, l[23]),
        o(P, Q),
        o(P, ae),
        o(P, W),
        o(P, x),
        o(P, H),
        o(P, U),
        o(P, le),
        Oe && Oe.m(P, null),
        o(f, J),
        o(f, ie),
        o(f, z),
        o(f, te),
        o(f, pe),
        o(pe, ue),
        o(f, he),
        o(f, de),
        o(de, _e),
        o(de, ne),
        o(de, $),
        nt($, l[2]),
        o(de, fe),
        o(de, we),
        o(de, j),
        o(de, F),
        nt(F, l[3]),
        o(de, ee),
        o(f, V),
        o(f, me),
        o(f, _t),
        o(f, Je),
        o(f, bt),
        o(f, ut),
        He && He.m(ut, null),
        o(f, it),
        o(f, Be),
        o(f, We),
        Ke && Ke.m(f, null),
        g(ke, Ie, Ue),
        g(ke, Ce, Ue),
        (De = !0),
        at ||
          ((Ve = [
            ce(d, "click", l[143]),
            ce(b, "click", l[144]),
            ce(q, "click", l[145]),
            ce(R, "input", l[146]),
            ce(R, "change", l[147]),
            ce(ae, "click", l[84]),
            ce(x, "click", l[89]),
            ce($, "input", l[148]),
            ce(F, "input", l[149]),
          ]),
          (at = !0)));
    },
    p(ke, Ue) {
      let nl = a;
      ((a = Xt(ke)),
        a === nl
          ? rt[a].p(ke, Ue)
          : (vs(),
            be(rt[nl], 1, 1, () => {
              rt[nl] = null;
            }),
            _s(),
            (i = rt[a]),
            i ? i.p(ke, Ue) : ((i = rt[a] = $t[a](ke)), i.c()),
            ve(i, 1),
            i.m(n, null)),
        (!De || (Ue[0] & 16777216 && v !== (v = Vs(ke[24] === "Substitution" ? "active" : "") + " svelte-1u6ecyq"))) && r(d, "class", v),
        (!De || (Ue[0] & 16777216 && y !== (y = Vs(ke[24] === "Insertion" ? "active" : "") + " svelte-1u6ecyq"))) && r(b, "class", y),
        (!De || (Ue[0] & 16777216 && N !== (N = Vs(ke[24] === "Deletion" ? "active" : "") + " svelte-1u6ecyq"))) && r(q, "class", N),
        ke[58].length > 0 ? (Xe ? Xe.p(ke, Ue) : ((Xe = Cr(ke)), Xe.c(), Xe.m(P, K))) : Xe && (Xe.d(1), (Xe = null)),
        Ue[0] & 8388608 && R.value !== ke[23] && nt(R, ke[23]),
        ke[58].length > 0 ? (Oe ? Oe.p(ke, Ue) : ((Oe = Tr(ke)), Oe.c(), Oe.m(P, null))) : Oe && (Oe.d(1), (Oe = null)),
        (!De || Ue[0] & 16777216) && Y !== (Y = ke[64][ke[24]] + "") && Z(ue, Y),
        Ue[0] & 4 && Us($.value) !== ke[2] && nt($, ke[2]),
        Ue[0] & 8 && Us(F.value) !== ke[3] && nt(F, ke[3]),
        ke[62].length > 0 ? (He ? He.p(ke, Ue) : ((He = Sr(ke)), He.c(), He.m(ut, null))) : He && (He.d(1), (He = null)),
        ke[24] != "Deletion" ? (Ke ? Ke.p(ke, Ue) : ((Ke = Mr(ke)), Ke.c(), Ke.m(f, null))) : Ke && (Ke.d(1), (Ke = null)));
    },
    i(ke) {
      De || (ve(i), (De = !0));
    },
    o(ke) {
      (be(i), (De = !1));
    },
    d(ke) {
      (ke && m(e),
        ke && m(t),
        ke && m(s),
        rt[a].d(),
        Xe && Xe.d(),
        Oe && Oe.d(),
        He && He.d(),
        Ke && Ke.d(),
        ke && m(Ie),
        ke && m(Ce),
        (at = !1),
        Zt(Ve));
    },
  };
}
function dr(l) {
  let e,
    t,
    s = l[62],
    n = [];
  for (let a = 0; a < s.length; a += 1) n[a] = pr(er(l, s, a));
  return {
    c() {
      ((e = C(`Warnings:
					`)),
        (t = c("ul")));
      for (let a = 0; a < n.length; a += 1) n[a].c();
      r(t, "class", "svelte-1u6ecyq");
    },
    m(a, i) {
      (g(a, e, i), g(a, t, i));
      for (let u = 0; u < n.length; u += 1) n[u] && n[u].m(t, null);
    },
    p(a, i) {
      if (i[2] & 1) {
        s = a[62];
        let u;
        for (u = 0; u < s.length; u += 1) {
          const f = er(a, s, u);
          n[u] ? n[u].p(f, i) : ((n[u] = pr(f)), n[u].c(), n[u].m(t, null));
        }
        for (; u < n.length; u += 1) n[u].d(1);
        n.length = s.length;
      }
    },
    d(a) {
      (a && m(e), a && m(t), cn(n, a));
    },
  };
}
function pr(l) {
  let e,
    t,
    s = l[62][l[259]] + "",
    n;
  return {
    c() {
      ((e = c("li")), (t = c("span")), (n = C(s)), r(t, "class", "warn svelte-1u6ecyq"), r(e, "class", "svelte-1u6ecyq"));
    },
    m(a, i) {
      (g(a, e, i), o(e, t), o(t, n));
    },
    p(a, i) {
      i[2] & 1 && s !== (s = a[62][a[259]] + "") && Z(n, s);
    },
    d(a) {
      a && m(e);
    },
  };
}
function fc(l) {
  let e;
  return {
    c() {
      ((e = c("div")),
        (e.innerHTML = '<strong class="svelte-1u6ecyq">Please enter a valid sequence in step 1.</strong>'),
        r(e, "class", "warn svelte-1u6ecyq"));
    },
    m(t, s) {
      g(t, e, s);
    },
    p: qe,
    i: qe,
    o: qe,
    d(t) {
      t && m(e);
    },
  };
}
function dc(l) {
  let e, t, s, n, a;
  const i = [mc, hc, pc],
    u = [];
  function f(h, d) {
    return (
      d[0] & 16385 && (e = null),
      e == null && (e = !!jt.positionError("ORF start", h[14], h[0].length)),
      e ? 0 : h[1].isCircular ? 1 : 2
    );
  }
  return (
    (t = f(l, [-1, -1, -1, -1, -1, -1, -1, -1, -1])),
    (s = u[t] = i[t](l)),
    {
      c() {
        (s.c(), (n = is()));
      },
      m(h, d) {
        (u[t].m(h, d), g(h, n, d), (a = !0));
      },
      p(h, d) {
        let p = t;
        ((t = f(h, d)),
          t === p
            ? u[t].p(h, d)
            : (vs(),
              be(u[p], 1, 1, () => {
                u[p] = null;
              }),
              _s(),
              (s = u[t]),
              s ? s.p(h, d) : ((s = u[t] = i[t](h)), s.c()),
              ve(s, 1),
              s.m(n.parentNode, n)));
      },
      i(h) {
        a || (ve(s), (a = !0));
      },
      o(h) {
        (be(s), (a = !1));
      },
      d(h) {
        (u[t].d(h), h && m(n));
      },
    }
  );
}
function pc(l) {
  let e, t;
  return (
    (e = new Ao({ props: { seq: l[0].slice(l[14] - 1), start: l[14] - 1, orfoffset: l[14], seqrec: l[1] } })),
    {
      c() {
        Re(e.$$.fragment);
      },
      m(s, n) {
        (Ge(e, s, n), (t = !0));
      },
      p(s, n) {
        const a = {};
        (n[0] & 16385 && (a.seq = s[0].slice(s[14] - 1)),
          n[0] & 16384 && (a.start = s[14] - 1),
          n[0] & 16384 && (a.orfoffset = s[14]),
          n[0] & 2 && (a.seqrec = s[1]),
          e.$set(a));
      },
      i(s) {
        t || (ve(e.$$.fragment, s), (t = !0));
      },
      o(s) {
        (be(e.$$.fragment, s), (t = !1));
      },
      d(s) {
        Fe(e, s);
      },
    }
  );
}
function hc(l) {
  let e, t;
  return (
    (e = new Ao({ props: { seq: l[0].slice(l[14] - 1) + l[0].slice(0, l[14] - 1), start: l[14] - 1, orfoffset: l[14], seqrec: l[1] } })),
    {
      c() {
        Re(e.$$.fragment);
      },
      m(s, n) {
        (Ge(e, s, n), (t = !0));
      },
      p(s, n) {
        const a = {};
        (n[0] & 16385 && (a.seq = s[0].slice(s[14] - 1) + s[0].slice(0, s[14] - 1)),
          n[0] & 16384 && (a.start = s[14] - 1),
          n[0] & 16384 && (a.orfoffset = s[14]),
          n[0] & 2 && (a.seqrec = s[1]),
          e.$set(a));
      },
      i(s) {
        t || (ve(e.$$.fragment, s), (t = !0));
      },
      o(s) {
        (be(e.$$.fragment, s), (t = !1));
      },
      d(s) {
        Fe(e, s);
      },
    }
  );
}
function mc(l) {
  let e,
    t,
    s = jt.positionError("ORF start", l[14], l[0].length) + "",
    n;
  return {
    c() {
      ((e = c("div")), (t = c("strong")), (n = C(s)), r(t, "class", "svelte-1u6ecyq"), r(e, "class", "warn svelte-1u6ecyq"));
    },
    m(a, i) {
      (g(a, e, i), o(e, t), o(t, n));
    },
    p(a, i) {
      i[0] & 16385 && s !== (s = jt.positionError("ORF start", a[14], a[0].length) + "") && Z(n, s);
    },
    i: qe,
    o: qe,
    d(a) {
      a && m(e);
    },
  };
}
function hr(l) {
  let e,
    t,
    s = l[62],
    n = [];
  for (let a = 0; a < s.length; a += 1) n[a] = mr(tr(l, s, a));
  return {
    c() {
      ((e = C(`Warnings:
				`)),
        (t = c("ul")));
      for (let a = 0; a < n.length; a += 1) n[a].c();
      r(t, "class", "svelte-1u6ecyq");
    },
    m(a, i) {
      (g(a, e, i), g(a, t, i));
      for (let u = 0; u < n.length; u += 1) n[u] && n[u].m(t, null);
    },
    p(a, i) {
      if (i[2] & 1) {
        s = a[62];
        let u;
        for (u = 0; u < s.length; u += 1) {
          const f = tr(a, s, u);
          n[u] ? n[u].p(f, i) : ((n[u] = mr(f)), n[u].c(), n[u].m(t, null));
        }
        for (; u < n.length; u += 1) n[u].d(1);
        n.length = s.length;
      }
    },
    d(a) {
      (a && m(e), a && m(t), cn(n, a));
    },
  };
}
function mr(l) {
  let e,
    t,
    s = l[62][l[259]] + "",
    n;
  return {
    c() {
      ((e = c("li")), (t = c("span")), (n = C(s)), r(t, "class", "warn svelte-1u6ecyq"), r(e, "class", "svelte-1u6ecyq"));
    },
    m(a, i) {
      (g(a, e, i), o(e, t), o(t, n));
    },
    p(a, i) {
      i[2] & 1 && s !== (s = a[62][a[259]] + "") && Z(n, s);
    },
    d(a) {
      a && m(e);
    },
  };
}
function gr(l) {
  let e, t, s, n, a, i, u, f, h, d;
  return (
    (i = new Rl({ props: { topic: "Generate NEBuilder HiFi primers" } })),
    i.$on("click", l[169]),
    {
      c() {
        ((e = c("div")),
          (t = c("input")),
          (s = S()),
          (n = c("label")),
          (n.textContent = "Generate primers for NEBuilder HiFi"),
          (a = C(`
				\xA0\xA0
				`)),
          Re(i.$$.fragment),
          (u = C(`   
				\xA0\xA0`)),
          r(t, "class", " svelte-1u6ecyq"),
          r(t, "type", "checkbox"),
          r(t, "id", "nebuildercheck"),
          r(n, "for", "nebuildercheck"),
          r(n, "class", "svelte-1u6ecyq"),
          r(e, "class", "svelte-1u6ecyq"));
      },
      m(p, v) {
        (g(p, e, v),
          o(e, t),
          (t.checked = l[16]),
          o(e, s),
          o(e, n),
          o(e, a),
          Ge(i, e, null),
          o(e, u),
          (f = !0),
          h || ((d = ce(t, "change", l[168])), (h = !0)));
      },
      p(p, v) {
        v[0] & 65536 && (t.checked = p[16]);
      },
      i(p) {
        f || (ve(i.$$.fragment, p), (f = !0));
      },
      o(p) {
        (be(i.$$.fragment, p), (f = !1));
      },
      d(p) {
        (p && m(e), Fe(i), (h = !1), d());
      },
    }
  );
}
function vr(l) {
  let e;
  return {
    c() {
      ((e = c("span")),
        (e.textContent =
          "Text selection within long sequences (>10kb) fails on Safari.  Use Chrome, Edge, or Firefox if you need this feature."),
        r(e, "class", "warn svelte-1u6ecyq"));
    },
    m(t, s) {
      g(t, e, s);
    },
    d(t) {
      t && m(e);
    },
  };
}
function _r(l) {
  let e,
    t,
    s = l[58].length + "",
    n,
    a;
  return {
    c() {
      ((e = c("span")),
        (t = c("strong")),
        (n = C(s)),
        (a = C(" match(es) found.")),
        r(t, "class", "svelte-1u6ecyq"),
        r(e, "class", "svelte-1u6ecyq"));
    },
    m(i, u) {
      (g(i, e, u), o(e, t), o(t, n), o(t, a));
    },
    p(i, u) {
      u[1] & 134217728 && s !== (s = i[58].length + "") && Z(n, s);
    },
    d(i) {
      i && m(e);
    },
  };
}
function br(l) {
  let e, t, s, n, a;
  return {
    c() {
      ((e = c("button")),
        (e.innerHTML = '<strong class="svelte-1u6ecyq">&lt;&lt; prev</strong>'),
        (t = S()),
        (s = c("button")),
        (s.innerHTML = '<strong class="svelte-1u6ecyq">next &gt;&gt;</strong>'),
        r(e, "class", "button btn btn-link svelte-1u6ecyq"),
        r(s, "class", "button btn btn-link svelte-1u6ecyq"));
    },
    m(i, u) {
      (g(i, e, u), g(i, t, u), g(i, s, u), n || ((a = [ce(e, "click", l[83]), ce(s, "click", l[82])]), (n = !0)));
    },
    p: qe,
    d(i) {
      (i && m(e), i && m(t), i && m(s), (n = !1), Zt(a));
    },
  };
}
function gc(l) {
  let e;
  return {
    c() {
      ((e = c("div")),
        (e.innerHTML = '<strong class="svelte-1u6ecyq">Please enter a valid sequence in step 1.</strong>'),
        r(e, "class", "warn svelte-1u6ecyq"));
    },
    m(t, s) {
      g(t, e, s);
    },
    p: qe,
    i: qe,
    o: qe,
    d(t) {
      t && m(e);
    },
  };
}
function vc(l) {
  let e,
    t,
    s = { seq: l[0], start: l[76](l[2], l[0].length), end: l[76](l[3], l[0].length) };
  return (
    (e = new Ga({ props: s })),
    l[156](e),
    {
      c() {
        Re(e.$$.fragment);
      },
      m(n, a) {
        (Ge(e, n, a), (t = !0));
      },
      p(n, a) {
        const i = {};
        (a[0] & 1 && (i.seq = n[0]),
          a[0] & 5 && (i.start = n[76](n[2], n[0].length)),
          a[0] & 9 && (i.end = n[76](n[3], n[0].length)),
          e.$set(i));
      },
      i(n) {
        t || (ve(e.$$.fragment, n), (t = !0));
      },
      o(n) {
        (be(e.$$.fragment, n), (t = !1));
      },
      d(n) {
        (l[156](null), Fe(e, n));
      },
    }
  );
}
function qr(l) {
  let e,
    t,
    s = l[62],
    n = [];
  for (let a = 0; a < s.length; a += 1) n[a] = wr(sr(l, s, a));
  return {
    c() {
      ((e = C(`Warnings:
					`)),
        (t = c("ul")));
      for (let a = 0; a < n.length; a += 1) n[a].c();
      r(t, "class", "svelte-1u6ecyq");
    },
    m(a, i) {
      (g(a, e, i), g(a, t, i));
      for (let u = 0; u < n.length; u += 1) n[u] && n[u].m(t, null);
    },
    p(a, i) {
      if (i[2] & 1) {
        s = a[62];
        let u;
        for (u = 0; u < s.length; u += 1) {
          const f = sr(a, s, u);
          n[u] ? n[u].p(f, i) : ((n[u] = wr(f)), n[u].c(), n[u].m(t, null));
        }
        for (; u < n.length; u += 1) n[u].d(1);
        n.length = s.length;
      }
    },
    d(a) {
      (a && m(e), a && m(t), cn(n, a));
    },
  };
}
function wr(l) {
  let e,
    t,
    s = l[62][l[259]] + "",
    n;
  return {
    c() {
      ((e = c("li")), (t = c("span")), (n = C(s)), r(t, "class", "warn svelte-1u6ecyq"), r(e, "class", "svelte-1u6ecyq"));
    },
    m(a, i) {
      (g(a, e, i), o(e, t), o(t, n));
    },
    p(a, i) {
      i[2] & 1 && s !== (s = a[62][a[259]] + "") && Z(n, s);
    },
    d(a) {
      a && m(e);
    },
  };
}
function yr(l) {
  let e,
    t,
    s = l[260][0] + "",
    n,
    a,
    i = l[260][1] + "",
    u;
  return {
    c() {
      ((e = c("option")),
        (t = c("code")),
        (n = C(s)),
        (a = C(" -- ")),
        (u = C(i)),
        r(t, "class", "svelte-1u6ecyq"),
        (e.__value = l[260][1]),
        (e.value = e.__value),
        r(e, "class", "svelte-1u6ecyq"));
    },
    m(f, h) {
      (g(f, e, h), o(e, t), o(t, n), o(t, a), o(t, u));
    },
    p: qe,
    d(f) {
      f && m(e);
    },
  };
}
function _c(l) {
  let e;
  return {
    c() {
      ((e = c("div")),
        (e.innerHTML = '<strong class="svelte-1u6ecyq">Please enter a valid sequence in step 1.</strong>'),
        r(e, "class", "warn svelte-1u6ecyq"));
    },
    m(t, s) {
      g(t, e, s);
    },
    p: qe,
    i: qe,
    o: qe,
    d(t) {
      t && m(e);
    },
  };
}
function bc(l) {
  let e,
    t,
    s,
    n = navigator.userAgent.includes("Safari") && !navigator.userAgent.includes("Chrome") && l[0].length > 1e4,
    a,
    i,
    u,
    f,
    h,
    d = n && Ar();
  function p(b) {
    l[141](b);
  }
  function v(b) {
    l[142](b);
  }
  let _ = { seq: l[0], start: l[76](l[2], l[0].length), end: l[76](l[3], l[0].length) };
  return (
    l[2] !== void 0 && (_.selstart = l[2]),
    l[3] !== void 0 && (_.selend = l[3]),
    (i = new ja({ props: _ })),
    l[140](i),
    dt.push(() => zt(i, "selstart", p)),
    dt.push(() => zt(i, "selend", v)),
    {
      c() {
        ((e = C("[optional] Click and drag in sequence to select mutagenesis region.")),
          (t = c("br")),
          (s = S()),
          d && d.c(),
          (a = S()),
          Re(i.$$.fragment),
          r(t, "class", "svelte-1u6ecyq"));
      },
      m(b, w) {
        (g(b, e, w), g(b, t, w), g(b, s, w), d && d.m(b, w), g(b, a, w), Ge(i, b, w), (h = !0));
      },
      p(b, w) {
        (w[0] & 1 && (n = navigator.userAgent.includes("Safari") && !navigator.userAgent.includes("Chrome") && b[0].length > 1e4),
          n ? d || ((d = Ar()), d.c(), d.m(a.parentNode, a)) : d && (d.d(1), (d = null)));
        const y = {};
        (w[0] & 1 && (y.seq = b[0]),
          w[0] & 5 && (y.start = b[76](b[2], b[0].length)),
          w[0] & 9 && (y.end = b[76](b[3], b[0].length)),
          !u && w[0] & 4 && ((u = !0), (y.selstart = b[2]), Wt(() => (u = !1))),
          !f && w[0] & 8 && ((f = !0), (y.selend = b[3]), Wt(() => (f = !1))),
          i.$set(y));
      },
      i(b) {
        h || (ve(i.$$.fragment, b), (h = !0));
      },
      o(b) {
        (be(i.$$.fragment, b), (h = !1));
      },
      d(b) {
        (b && m(e), b && m(t), b && m(s), d && d.d(b), b && m(a), l[140](null), Fe(i, b));
      },
    }
  );
}
function Ar(l) {
  let e;
  return {
    c() {
      ((e = c("span")),
        (e.innerHTML =
          'Text selection within long sequences (&gt;10kb) fails on Safari. <br class="svelte-1u6ecyq"/>Use Chrome, Edge, or Firefox if you need this feature.'),
        r(e, "class", "warn svelte-1u6ecyq"));
    },
    m(t, s) {
      g(t, e, s);
    },
    d(t) {
      t && m(e);
    },
  };
}
function Cr(l) {
  let e,
    t,
    s = l[58].length + "",
    n,
    a;
  return {
    c() {
      ((e = c("span")),
        (t = c("strong")),
        (n = C(s)),
        (a = C(" match(es).")),
        r(t, "class", "svelte-1u6ecyq"),
        r(e, "class", "svelte-1u6ecyq"));
    },
    m(i, u) {
      (g(i, e, u), o(e, t), o(t, n), o(t, a));
    },
    p(i, u) {
      u[1] & 134217728 && s !== (s = i[58].length + "") && Z(n, s);
    },
    d(i) {
      i && m(e);
    },
  };
}
function Tr(l) {
  let e, t, s, n, a;
  return {
    c() {
      ((e = c("button")),
        (e.innerHTML = '<strong class="svelte-1u6ecyq">&lt;&lt; prev</strong>'),
        (t = S()),
        (s = c("button")),
        (s.innerHTML = '<strong class="svelte-1u6ecyq">next &gt;&gt;</strong>'),
        r(e, "class", "button btn btn-link svelte-1u6ecyq"),
        r(s, "class", "button btn btn-link svelte-1u6ecyq"));
    },
    m(i, u) {
      (g(i, e, u), g(i, t, u), g(i, s, u), n || ((a = [ce(e, "click", l[86]), ce(s, "click", l[85])]), (n = !0)));
    },
    p: qe,
    d(i) {
      (i && m(e), i && m(t), i && m(s), (n = !1), Zt(a));
    },
  };
}
function Sr(l) {
  let e,
    t,
    s = l[62],
    n = [];
  for (let a = 0; a < s.length; a += 1) n[a] = kr(nr(l, s, a));
  return {
    c() {
      ((e = C(`Warnings:
						`)),
        (t = c("ul")));
      for (let a = 0; a < n.length; a += 1) n[a].c();
      r(t, "class", "svelte-1u6ecyq");
    },
    m(a, i) {
      (g(a, e, i), g(a, t, i));
      for (let u = 0; u < n.length; u += 1) n[u] && n[u].m(t, null);
    },
    p(a, i) {
      if (i[2] & 1) {
        s = a[62];
        let u;
        for (u = 0; u < s.length; u += 1) {
          const f = nr(a, s, u);
          n[u] ? n[u].p(f, i) : ((n[u] = kr(f)), n[u].c(), n[u].m(t, null));
        }
        for (; u < n.length; u += 1) n[u].d(1);
        n.length = s.length;
      }
    },
    d(a) {
      (a && m(e), a && m(t), cn(n, a));
    },
  };
}
function kr(l) {
  let e,
    t,
    s = l[62][l[259]] + "",
    n;
  return {
    c() {
      ((e = c("li")), (t = c("span")), (n = C(s)), r(t, "class", "warn svelte-1u6ecyq"), r(e, "class", "svelte-1u6ecyq"));
    },
    m(a, i) {
      (g(a, e, i), o(e, t), o(t, n));
    },
    p(a, i) {
      i[2] & 1 && s !== (s = a[62][a[259]] + "") && Z(n, s);
    },
    d(a) {
      a && m(e);
    },
  };
}
function Mr(l) {
  let e, t, s, n, a, i, u, f, h, d, p, v, _, b, w, y, A, q, T, N, M, k, E, P, G;
  return {
    c() {
      ((e = c("div")),
        (t = c("label")),
        (s = C("Desired Sequence (ambiguity allowed) -- leave blank if deleting target.")),
        (n = c("br")),
        (a = S()),
        (i = c("input")),
        (u = C(`
	                	\xA0\xA0
	                	`)),
        (f = c("button")),
        (f.innerHTML = '<strong class="svelte-1u6ecyq">clear X</strong>'),
        (h = S()),
        (d = c("select")),
        (p = c("option")),
        (p.innerHTML = '<strong class="svelte-1u6ecyq">Common Peptide Tags</strong>'),
        (v = c("option")),
        (v.innerHTML = `<span class="tagname svelte-1u6ecyq">6x His</span>
		                        --
		                        <span class="tagaa svelte-1u6ecyq">HHHHHH</span> 
		                    `),
        (_ = c("option")),
        (_.innerHTML = `<span class="tagname svelte-1u6ecyq">MYC</span>
		                        --
		                        <span class="tagaa svelte-1u6ecyq">EQKLISEEDL</span> 
		                    `),
        (b = c("option")),
        (b.innerHTML = `<span class="tagname svelte-1u6ecyq">FLAG</span>
		                        --
		                        <span class="tagaa svelte-1u6ecyq">DYKDDDDK</span> 
		                    `),
        (w = c("option")),
        (w.innerHTML = `<span class="tagname svelte-1u6ecyq">HA</span>
		                        --
		                        <span class="tagaa svelte-1u6ecyq">YPYDVPDYA</span> 
		                    `),
        (y = c("option")),
        (y.innerHTML = `<span class="tagname svelte-1u6ecyq">Strep</span>
		                        --
		                        <span class="tagaa svelte-1u6ecyq">WSHPQFEK</span> 
		                    `),
        (A = c("option")),
        (A.innerHTML = `<span class="tagname svelte-1u6ecyq">Xpress</span>
		                        --
		                        <span class="tagaa svelte-1u6ecyq">DLYDDDDK</span> 
		                    `),
        (q = c("option")),
        (q.innerHTML = `<span class="tagname svelte-1u6ecyq">AviTag</span>
		                        --
		                        <span class="tagaa svelte-1u6ecyq">GLNDIFEAQKIEWH</span> 
		                    `),
        (T = c("option")),
        (T.innerHTML = `<span class="tagname svelte-1u6ecyq">V5</span>
		                        --
		                        <span class="tagaa svelte-1u6ecyq">GKPIPNPLLGLDST</span> 
		                    `),
        (N = c("option")),
        (N.innerHTML = `<span class="tagname svelte-1u6ecyq">NLS</span>
		                        --
		                        <span class="tagaa svelte-1u6ecyq">PKKKRKV</span> 
		                    `),
        (M = c("option")),
        (M.innerHTML = `<span class="tagname svelte-1u6ecyq">SPOT</span>
		                        --
		                        <span class="tagaa svelte-1u6ecyq">PDRVRAVSHWSS</span>`),
        (k = S()),
        (E = c("div")),
        (E.textContent = "\xA0"),
        r(n, "class", "svelte-1u6ecyq"),
        r(i, "type", "text"),
        r(i, "id", "repseq"),
        r(i, "class", "longinput svelte-1u6ecyq"),
        r(t, "for", "repseq"),
        r(t, "class", "svelte-1u6ecyq"),
        r(f, "class", "button btn btn-link svelte-1u6ecyq"),
        (p.__value = ""),
        (p.value = p.__value),
        (p.selected = "selected"),
        r(p, "class", "svelte-1u6ecyq"),
        (v.__value = "CAT CAT CAT CAC CAC CAC"),
        (v.value = v.__value),
        r(v, "class", "svelte-1u6ecyq"),
        (_.__value = "GAA CAG AAA CTG ATT AGC GAA GAA GAT CTG"),
        (_.value = _.__value),
        r(_, "class", "svelte-1u6ecyq"),
        (b.__value = "GAT TAT AAA GAT GAT GAT GAT AAA"),
        (b.value = b.__value),
        r(b, "class", "svelte-1u6ecyq"),
        (w.__value = "TAT CCG TAT GAT GTG CCG GAT TAT GCG"),
        (w.value = w.__value),
        r(w, "class", "svelte-1u6ecyq"),
        (y.__value = "TGG AGC CAT CCG CAG TTT GAA AAA"),
        (y.value = y.__value),
        r(y, "class", "svelte-1u6ecyq"),
        (A.__value = "GAT CTG TAT GAT GAT GAT GAT AAA"),
        (A.value = A.__value),
        r(A, "class", "svelte-1u6ecyq"),
        (q.__value = "GGC CTG AAC GAT ATT TTT GAA GCG CAG AAA ATT GAA TGG CAT GAA"),
        (q.value = q.__value),
        r(q, "class", "svelte-1u6ecyq"),
        (T.__value = "GGC AAA CCG ATT CCG AAC CCG CTG CTG GGC CTG GAT AGC ACC"),
        (T.value = T.__value),
        r(T, "class", "svelte-1u6ecyq"),
        (N.__value = "CCG AAA AAA AAA CGC AAA GTG"),
        (N.value = N.__value),
        r(N, "class", "svelte-1u6ecyq"),
        (M.__value = " CCG GAC CGT GTC CGT GCA GTC TCT CAC TGG TCC TCG "),
        (M.value = M.__value),
        r(M, "class", "svelte-1u6ecyq"),
        r(d, "id", "peptag"),
        r(d, "aria-label", "Common peptide tags"),
        r(d, "class", "dropdown neb-dropdown svelte-1u6ecyq"),
        l[5] === void 0 && vn(() => l[151].call(d)),
        r(E, "id", "notesindicator"),
        r(E, "class", "svelte-1u6ecyq"),
        r(e, "id", "replacement"),
        r(e, "class", "svelte-1u6ecyq"));
    },
    m(O, K) {
      (g(O, e, K),
        o(e, t),
        o(t, s),
        o(t, n),
        o(t, a),
        o(t, i),
        nt(i, l[4]),
        o(e, u),
        o(e, f),
        o(e, h),
        o(e, d),
        o(d, p),
        o(d, v),
        o(d, _),
        o(d, b),
        o(d, w),
        o(d, y),
        o(d, A),
        o(d, q),
        o(d, T),
        o(d, N),
        o(d, M),
        gn(d, l[5], !0),
        o(e, k),
        o(e, E),
        P || ((G = [ce(i, "input", l[150]), ce(i, "blur", l[80]), ce(f, "click", l[90]), ce(d, "change", l[151])]), (P = !0)));
    },
    p(O, K) {
      (K[0] & 16 && i.value !== O[4] && nt(i, O[4]), K[0] & 32 && gn(d, O[5]));
    },
    d(O) {
      (O && m(e), (P = !1), Zt(G));
    },
  };
}
function Er(l) {
  let e, t, s, n, a, i, u, f, h, d, p, v, _, b, w, y, A, q, T, N;
  ((u = new Rl({ props: { topic: "Confine mutations to primer tails" } })), u.$on("click", l[175]));
  let M = l[8] && Nr();
  ((w = new Rl({ props: { topic: "Preserve degenerate bases" } })), w.$on("click", l[177]));
  let k = l[12] == "codon" && Pr(l);
  return {
    c() {
      ((e = c("div")),
        (t = c("input")),
        (n = S()),
        (a = c("label")),
        (a.textContent = "Confine mutations to primer 5\u2032 tails"),
        (i = C(`
		        \xA0\xA0
		   `)),
        Re(u.$$.fragment),
        (f = S()),
        M && M.c(),
        (h = S()),
        (d = c("div")),
        (p = c("input")),
        (v = S()),
        (_ = c("label")),
        (_.textContent = "Preserve degenerate bases"),
        (b = C(`
		        \xA0\xA0
		   `)),
        Re(w.$$.fragment),
        (y = S()),
        k && k.c(),
        (A = is()),
        r(t, "class", " svelte-1u6ecyq"),
        r(t, "type", "checkbox"),
        r(t, "id", "useTailsOnly"),
        (t.disabled = l[8]),
        r(t, "aria-describedby", (s = l[8] ? "useTailsOnlyLockedNote" : void 0)),
        r(a, "for", "useTailsOnly"),
        r(a, "class", "svelte-1u6ecyq"),
        r(e, "class", "svelte-1u6ecyq"),
        r(p, "class", " svelte-1u6ecyq"),
        r(p, "type", "checkbox"),
        r(p, "id", "keepDegenBases"),
        r(_, "for", "keepDegenBases"),
        r(_, "class", "svelte-1u6ecyq"),
        r(d, "class", "svelte-1u6ecyq"));
    },
    m(E, P) {
      (g(E, e, P),
        o(e, t),
        (t.checked = l[7]),
        o(e, n),
        o(e, a),
        o(e, i),
        Ge(u, e, null),
        o(e, f),
        M && M.m(e, null),
        g(E, h, P),
        g(E, d, P),
        o(d, p),
        (p.checked = l[8]),
        o(d, v),
        o(d, _),
        o(d, b),
        Ge(w, d, null),
        g(E, y, P),
        k && k.m(E, P),
        g(E, A, P),
        (q = !0),
        T || ((N = [ce(t, "change", l[174]), ce(p, "change", l[176]), ce(p, "change", l[73])]), (T = !0)));
    },
    p(E, P) {
      ((!q || P[0] & 256) && (t.disabled = E[8]),
        (!q || (P[0] & 256 && s !== (s = E[8] ? "useTailsOnlyLockedNote" : void 0))) && r(t, "aria-describedby", s),
        P[0] & 128 && (t.checked = E[7]),
        E[8] ? M || ((M = Nr()), M.c(), M.m(e, null)) : M && (M.d(1), (M = null)),
        P[0] & 256 && (p.checked = E[8]),
        E[12] == "codon"
          ? k
            ? (k.p(E, P), P[0] & 4096 && ve(k, 1))
            : ((k = Pr(E)), k.c(), ve(k, 1), k.m(A.parentNode, A))
          : k &&
            (vs(),
            be(k, 1, 1, () => {
              k = null;
            }),
            _s()));
    },
    i(E) {
      q || (ve(u.$$.fragment, E), ve(w.$$.fragment, E), ve(k), (q = !0));
    },
    o(E) {
      (be(u.$$.fragment, E), be(w.$$.fragment, E), be(k), (q = !1));
    },
    d(E) {
      (E && m(e), Fe(u), M && M.d(), E && m(h), E && m(d), Fe(w), E && m(y), k && k.d(E), E && m(A), (T = !1), Zt(N));
    },
  };
}
function Nr(l) {
  let e;
  return {
    c() {
      ((e = c("span")),
        (e.textContent = "(required while degenerate bases are preserved)"),
        r(e, "id", "useTailsOnlyLockedNote"),
        r(e, "class", "hint svelte-1u6ecyq"));
    },
    m(t, s) {
      g(t, e, s);
    },
    d(t) {
      t && m(e);
    },
  };
}
function Pr(l) {
  let e, t, s, n, a, i, u, f, h;
  return (
    (i = new Rl({ props: { topic: "Merge multiple point mutations" } })),
    i.$on("click", l[179]),
    {
      c() {
        ((e = c("div")),
          (t = c("input")),
          (s = S()),
          (n = c("label")),
          (n.textContent = "Merge mutations"),
          (a = C(`
		        \xA0\xA0
		   `)),
          Re(i.$$.fragment),
          r(t, "class", " svelte-1u6ecyq"),
          r(t, "type", "checkbox"),
          r(t, "id", "mergeMutations"),
          r(n, "for", "mergeMutations"),
          r(n, "class", "svelte-1u6ecyq"),
          r(e, "class", "svelte-1u6ecyq"));
      },
      m(d, p) {
        (g(d, e, p),
          o(e, t),
          (t.checked = l[10]),
          o(e, s),
          o(e, n),
          o(e, a),
          Ge(i, e, null),
          (u = !0),
          f || ((h = [ce(t, "change", l[178]), ce(t, "change", l[74])]), (f = !0)));
      },
      p(d, p) {
        p[0] & 1024 && (t.checked = d[10]);
      },
      i(d) {
        u || (ve(i.$$.fragment, d), (u = !0));
      },
      o(d) {
        (be(i.$$.fragment, d), (u = !1));
      },
      d(d) {
        (d && m(e), Fe(i), (f = !1), Zt(h));
      },
    }
  );
}
function Lr(l) {
  let e;
  return {
    c() {
      ((e = c("span")), (e.textContent = "Too low.  Using 15 nt."), r(e, "class", "warn svelte-1u6ecyq"));
    },
    m(t, s) {
      g(t, e, s);
    },
    d(t) {
      t && m(e);
    },
  };
}
function Gr(l) {
  let e;
  return {
    c() {
      ((e = c("span")), (e.textContent = "Too low.  Using 55\xB0C."), r(e, "class", "warn svelte-1u6ecyq"));
    },
    m(t, s) {
      g(t, e, s);
    },
    d(t) {
      t && m(e);
    },
  };
}
function Fr(l) {
  let e;
  return {
    c() {
      ((e = c("span")), (e.textContent = "Too high.  Using 72\xB0C."), r(e, "class", "warn svelte-1u6ecyq"));
    },
    m(t, s) {
      g(t, e, s);
    },
    d(t) {
      t && m(e);
    },
  };
}
function Rr(l) {
  let e;
  return {
    c() {
      ((e = c("div")), (e.textContent = "Please enter a new sequence."), r(e, "class", "svelte-1u6ecyq"));
    },
    m(t, s) {
      g(t, e, s);
    },
    d(t) {
      t && m(e);
    },
  };
}
function qc(l) {
  let e;
  return {
    c() {
      ((e = c("div")), (e.textContent = "No primers have been generated."), r(e, "class", "svelte-1u6ecyq"));
    },
    m(t, s) {
      g(t, e, s);
    },
    p: qe,
    i: qe,
    o: qe,
    d(t) {
      t && m(e);
    },
  };
}
function wc(l) {
  let e,
    t,
    s,
    n,
    a,
    i,
    u,
    f,
    h,
    d,
    p,
    v,
    _,
    b,
    w,
    y,
    A,
    q,
    T,
    N,
    M = l[9].length > 1 && Ir(l),
    k = !l[18] && Dr(l),
    E = l[18] && Hr(l),
    P = !l[18] && Ur(l);
  const G = [Tc, Cc],
    O = [];
  function K(D, L) {
    return D[18] ? 0 : 1;
  }
  return (
    (w = K(l)),
    (y = O[w] = G[w](l)),
    {
      c() {
        ((e = c("div")),
          (t = c("div")),
          (s = c("span")),
          (s.textContent = "Summary Table:"),
          (n = S()),
          (a = c("input")),
          (i = C(`
					\xA0\xA0
					`)),
          (u = c("button")),
          (u.textContent = "Download"),
          (f = S()),
          M && M.c(),
          (h = S()),
          k && k.c(),
          (d = S()),
          E && E.c(),
          (p = S()),
          P && P.c(),
          (v = S()),
          (_ = c("h3")),
          (_.textContent = "Required Primers"),
          (b = S()),
          y.c(),
          (A = is()),
          r(s, "class", "svelte-1u6ecyq"),
          r(a, "type", "text"),
          r(a, "placeholder", l[31]),
          r(a, "aria-label", "Summary table filename"),
          r(a, "class", "svelte-1u6ecyq"),
          r(u, "class", "button btn btn-link svelte-1u6ecyq"),
          r(t, "class", "rowf svelte-1u6ecyq"),
          r(e, "id", "downloads-container"),
          r(e, "class", "svelte-1u6ecyq"),
          r(_, "class", "svelte-1u6ecyq"));
      },
      m(D, L) {
        (g(D, e, L),
          o(e, t),
          o(t, s),
          o(t, n),
          o(t, a),
          nt(a, l[31]),
          o(t, i),
          o(t, u),
          o(e, f),
          M && M.m(e, null),
          o(e, h),
          k && k.m(e, null),
          o(e, d),
          E && E.m(e, null),
          g(D, p, L),
          P && P.m(D, L),
          g(D, v, L),
          g(D, _, L),
          g(D, b, L),
          O[w].m(D, L),
          g(D, A, L),
          (q = !0),
          T || ((N = [ce(a, "input", l[186]), ce(u, "click", l[71])]), (T = !0)));
      },
      p(D, L) {
        ((!q || L[1] & 1) && r(a, "placeholder", D[31]),
          L[1] & 1 && a.value !== D[31] && nt(a, D[31]),
          D[9].length > 1 ? (M ? M.p(D, L) : ((M = Ir(D)), M.c(), M.m(e, h))) : M && (M.d(1), (M = null)),
          D[18] ? k && (k.d(1), (k = null)) : k ? k.p(D, L) : ((k = Dr(D)), k.c(), k.m(e, d)),
          D[18]
            ? E
              ? (E.p(D, L), L[0] & 262144 && ve(E, 1))
              : ((E = Hr(D)), E.c(), ve(E, 1), E.m(e, null))
            : E &&
              (vs(),
              be(E, 1, 1, () => {
                E = null;
              }),
              _s()),
          D[18]
            ? P &&
              (vs(),
              be(P, 1, 1, () => {
                P = null;
              }),
              _s())
            : P
              ? (P.p(D, L), L[0] & 262144 && ve(P, 1))
              : ((P = Ur(D)), P.c(), ve(P, 1), P.m(v.parentNode, v)));
        let R = w;
        ((w = K(D)),
          w === R
            ? O[w].p(D, L)
            : (vs(),
              be(O[R], 1, 1, () => {
                O[R] = null;
              }),
              _s(),
              (y = O[w]),
              y ? y.p(D, L) : ((y = O[w] = G[w](D)), y.c()),
              ve(y, 1),
              y.m(A.parentNode, A)));
      },
      i(D) {
        q || (ve(E), ve(P), ve(y), (q = !0));
      },
      o(D) {
        (be(E), be(P), be(y), (q = !1));
      },
      d(D) {
        (D && m(e),
          M && M.d(),
          k && k.d(),
          E && E.d(),
          D && m(p),
          P && P.d(D),
          D && m(v),
          D && m(_),
          D && m(b),
          O[w].d(D),
          D && m(A),
          (T = !1),
          Zt(N));
      },
    }
  );
}
function Ir(l) {
  let e, t, s, n, a, i, u, f;
  return {
    c() {
      ((e = c("div")),
        (t = c("span")),
        (t.textContent = "All Primers:"),
        (s = S()),
        (n = c("input")),
        (a = C(`
					\xA0\xA0
					`)),
        (i = c("button")),
        (i.textContent = "Download"),
        r(t, "class", "svelte-1u6ecyq"),
        r(n, "type", "text"),
        r(n, "placeholder", l[27]),
        r(n, "aria-label", "All primers filename"),
        r(n, "class", "svelte-1u6ecyq"),
        r(i, "class", "button btn btn-link svelte-1u6ecyq"),
        r(e, "class", "rowf svelte-1u6ecyq"));
    },
    m(h, d) {
      (g(h, e, d),
        o(e, t),
        o(e, s),
        o(e, n),
        nt(n, l[27]),
        o(e, a),
        o(e, i),
        u || ((f = [ce(n, "input", l[187]), ce(i, "click", l[69])]), (u = !0)));
    },
    p(h, d) {
      (d[0] & 134217728 && r(n, "placeholder", h[27]), d[0] & 134217728 && n.value !== h[27] && nt(n, h[27]));
    },
    d(h) {
      (h && m(e), (u = !1), Zt(f));
    },
  };
}
function Dr(l) {
  let e,
    t,
    s,
    n,
    a,
    i,
    u,
    f,
    h,
    d,
    p,
    v,
    _,
    b,
    w,
    y,
    A,
    q,
    T = l[9].length > 1 && Or(l),
    N = l[32].length > 0 && jr(l);
  return {
    c() {
      ((e = c("div")),
        (t = c("span")),
        (t.textContent = "Selected Primers:"),
        (s = S()),
        (n = c("input")),
        (a = C(`
					\xA0\xA0
					`)),
        (i = c("button")),
        (i.textContent = "Download"),
        (u = S()),
        T && T.c(),
        (f = S()),
        (h = c("div")),
        (d = c("span")),
        (d.textContent = "Selected Sequence:"),
        (p = S()),
        (v = c("input")),
        (_ = C(`
					\xA0\xA0
					`)),
        (b = c("button")),
        (b.textContent = "Download"),
        (w = S()),
        N && N.c(),
        (y = is()),
        r(t, "class", "svelte-1u6ecyq"),
        r(n, "type", "text"),
        r(n, "placeholder", l[28]),
        r(n, "aria-label", "Selected primers filename"),
        r(n, "class", "svelte-1u6ecyq"),
        r(i, "class", "button btn btn-link svelte-1u6ecyq"),
        r(e, "class", "rowf svelte-1u6ecyq"),
        r(d, "class", "svelte-1u6ecyq"),
        r(v, "type", "text"),
        r(v, "placeholder", l[29]),
        r(v, "aria-label", "Selected sequence filename"),
        r(v, "class", "svelte-1u6ecyq"),
        r(b, "class", "button btn btn-link svelte-1u6ecyq"),
        r(h, "class", "rowf svelte-1u6ecyq"));
    },
    m(M, k) {
      (g(M, e, k),
        o(e, t),
        o(e, s),
        o(e, n),
        nt(n, l[28]),
        o(e, a),
        o(e, i),
        g(M, u, k),
        T && T.m(M, k),
        g(M, f, k),
        g(M, h, k),
        o(h, d),
        o(h, p),
        o(h, v),
        nt(v, l[29]),
        o(h, _),
        o(h, b),
        g(M, w, k),
        N && N.m(M, k),
        g(M, y, k),
        A || ((q = [ce(n, "input", l[188]), ce(i, "click", l[70]), ce(v, "input", l[190]), ce(b, "click", l[67])]), (A = !0)));
    },
    p(M, k) {
      (k[0] & 268435456 && r(n, "placeholder", M[28]),
        k[0] & 268435456 && n.value !== M[28] && nt(n, M[28]),
        M[9].length > 1 ? (T ? T.p(M, k) : ((T = Or(M)), T.c(), T.m(f.parentNode, f))) : T && (T.d(1), (T = null)),
        k[0] & 536870912 && r(v, "placeholder", M[29]),
        k[0] & 536870912 && v.value !== M[29] && nt(v, M[29]),
        M[32].length > 0 ? (N ? N.p(M, k) : ((N = jr(M)), N.c(), N.m(y.parentNode, y))) : N && (N.d(1), (N = null)));
    },
    d(M) {
      (M && m(e), M && m(u), T && T.d(M), M && m(f), M && m(h), M && m(w), N && N.d(M), M && m(y), (A = !1), Zt(q));
    },
  };
}
function Or(l) {
  let e, t, s, n, a, i, u, f;
  return {
    c() {
      ((e = c("div")),
        (t = c("span")),
        (t.textContent = "All Sequences:"),
        (s = S()),
        (n = c("input")),
        (a = C(`
					\xA0\xA0
					`)),
        (i = c("button")),
        (i.textContent = "Download"),
        r(t, "class", "svelte-1u6ecyq"),
        r(n, "type", "text"),
        r(n, "placeholder", l[30]),
        r(n, "aria-label", "All sequences filename"),
        r(n, "class", "svelte-1u6ecyq"),
        r(i, "class", "button btn btn-link svelte-1u6ecyq"),
        r(e, "class", "rowf svelte-1u6ecyq"));
    },
    m(h, d) {
      (g(h, e, d),
        o(e, t),
        o(e, s),
        o(e, n),
        nt(n, l[30]),
        o(e, a),
        o(e, i),
        u || ((f = [ce(n, "input", l[189]), ce(i, "click", l[68])]), (u = !0)));
    },
    p(h, d) {
      (d[0] & 1073741824 && r(n, "placeholder", h[30]), d[0] & 1073741824 && n.value !== h[30] && nt(n, h[30]));
    },
    d(h) {
      (h && m(e), (u = !1), Zt(f));
    },
  };
}
function jr(l) {
  let e,
    t = l[32],
    s = [];
  for (let n = 0; n < t.length; n += 1) s[n] = Br($i(l, t, n));
  return {
    c() {
      e = c("ul");
      for (let n = 0; n < s.length; n += 1) s[n].c();
      r(e, "class", "svelte-1u6ecyq");
    },
    m(n, a) {
      g(n, e, a);
      for (let i = 0; i < s.length; i += 1) s[i] && s[i].m(e, null);
    },
    p(n, a) {
      if (a[1] & 2) {
        t = n[32];
        let i;
        for (i = 0; i < t.length; i += 1) {
          const u = $i(n, t, i);
          s[i] ? s[i].p(u, a) : ((s[i] = Br(u)), s[i].c(), s[i].m(e, null));
        }
        for (; i < s.length; i += 1) s[i].d(1);
        s.length = t.length;
      }
    },
    d(n) {
      (n && m(e), cn(s, n));
    },
  };
}
function Br(l) {
  let e,
    t,
    s = l[255] + "",
    n;
  return {
    c() {
      ((e = c("li")), (t = c("span")), (n = C(s)), r(t, "class", "warn svelte-1u6ecyq"), r(e, "class", "svelte-1u6ecyq"));
    },
    m(a, i) {
      (g(a, e, i), o(e, t), o(t, n));
    },
    p(a, i) {
      i[1] & 2 && s !== (s = a[255] + "") && Z(n, s);
    },
    d(a) {
      a && m(e);
    },
  };
}
function Hr(l) {
  let e, t, s, n, a, i, u, f, h, d;
  return (
    (u = new Rl({ props: { topic: "Use NEBuilder Calculator" } })),
    u.$on("click", l[191]),
    {
      c() {
        ((e = c("br")),
          (t = c("br")),
          (s = S()),
          (n = c("div")),
          (a = c("button")),
          (a.textContent = "Generate protocol in NEBuilder Calculator"),
          (i = S()),
          Re(u.$$.fragment),
          r(e, "class", "svelte-1u6ecyq"),
          r(t, "class", "svelte-1u6ecyq"),
          r(a, "class", "button btn btn-link svelte-1u6ecyq"),
          r(n, "class", "rowf svelte-1u6ecyq"));
      },
      m(p, v) {
        (g(p, e, v),
          g(p, t, v),
          g(p, s, v),
          g(p, n, v),
          o(n, a),
          o(n, i),
          Ge(u, n, null),
          (f = !0),
          h || ((d = ce(a, "click", l[72])), (h = !0)));
      },
      p: qe,
      i(p) {
        f || (ve(u.$$.fragment, p), (f = !0));
      },
      o(p) {
        (be(u.$$.fragment, p), (f = !1));
      },
      d(p) {
        (p && m(e), p && m(t), p && m(s), p && m(n), Fe(u), (h = !1), d());
      },
    }
  );
}
function Ur(l) {
  let e, t, s, n;
  const a = [Ac, yc],
    i = [];
  function u(f, h) {
    return f[9][f[26]] ? 0 : 1;
  }
  return (
    (e = u(l)),
    (t = i[e] = a[e](l)),
    {
      c() {
        (t.c(), (s = is()));
      },
      m(f, h) {
        (i[e].m(f, h), g(f, s, h), (n = !0));
      },
      p(f, h) {
        let d = e;
        ((e = u(f)),
          e === d
            ? i[e].p(f, h)
            : (vs(),
              be(i[d], 1, 1, () => {
                i[d] = null;
              }),
              _s(),
              (t = i[e]),
              t ? t.p(f, h) : ((t = i[e] = a[e](f)), t.c()),
              ve(t, 1),
              t.m(s.parentNode, s)));
      },
      i(f) {
        n || (ve(t), (n = !0));
      },
      o(f) {
        (be(t), (n = !1));
      },
      d(f) {
        (i[e].d(f), f && m(s));
      },
    }
  );
}
function yc(l) {
  let e;
  return {
    c() {
      ((e = c("div")),
        (e.textContent = "Select a primer result (below, View) to see a detailed view."),
        r(e, "class", "warn svelte-1u6ecyq"));
    },
    m(t, s) {
      g(t, e, s);
    },
    p: qe,
    i: qe,
    o: qe,
    d(t) {
      t && m(e);
    },
  };
}
function Ac(l) {
  let e,
    t,
    s = l[9][l[26]].idprefix + "",
    n,
    a,
    i = l[9][l[26]].pairIndex + "",
    u,
    f,
    h,
    d,
    p;
  return (
    (d = new Ca({
      props: { primerpair: l[9][l[26]], seqop: l[9][l[26]].seqop, repseq: l[9][l[26]].repseq, show: [1, 2, 3], nohi: l[59] },
    })),
    {
      c() {
        ((e = c("br")),
          (t = S()),
          (n = C(s)),
          (a = C("_")),
          (u = C(i)),
          (f = c("br")),
          (h = S()),
          Re(d.$$.fragment),
          r(e, "class", "svelte-1u6ecyq"),
          r(f, "class", "svelte-1u6ecyq"));
      },
      m(v, _) {
        (g(v, e, _), g(v, t, _), g(v, n, _), g(v, a, _), g(v, u, _), g(v, f, _), g(v, h, _), Ge(d, v, _), (p = !0));
      },
      p(v, _) {
        ((!p || _[0] & 67109376) && s !== (s = v[9][v[26]].idprefix + "") && Z(n, s),
          (!p || _[0] & 67109376) && i !== (i = v[9][v[26]].pairIndex + "") && Z(u, i));
        const b = {};
        (_[0] & 67109376 && (b.primerpair = v[9][v[26]]),
          _[0] & 67109376 && (b.seqop = v[9][v[26]].seqop),
          _[0] & 67109376 && (b.repseq = v[9][v[26]].repseq),
          _[1] & 268435456 && (b.nohi = v[59]),
          d.$set(b));
      },
      i(v) {
        p || (ve(d.$$.fragment, v), (p = !0));
      },
      o(v) {
        (be(d.$$.fragment, v), (p = !1));
      },
      d(v) {
        (v && m(e), v && m(t), v && m(n), v && m(a), v && m(u), v && m(f), v && m(h), Fe(d, v));
      },
    }
  );
}
function Cc(l) {
  let e, t, s, n, a, i;
  function u(h) {
    l[193](h);
  }
  let f = { primers: l[9] };
  return (
    l[26] !== void 0 && (f.selectedtxl = l[26]),
    (t = new ha({ props: f })),
    dt.push(() => zt(t, "selectedtxl", u)),
    {
      c() {
        ((e = c("div")),
          Re(t.$$.fragment),
          (n = S()),
          (a = c("br")),
          r(e, "id", "primertablecontainer"),
          r(e, "class", "svelte-1u6ecyq"),
          r(a, "class", "svelte-1u6ecyq"));
      },
      m(h, d) {
        (g(h, e, d), Ge(t, e, null), g(h, n, d), g(h, a, d), (i = !0));
      },
      p(h, d) {
        const p = {};
        (d[0] & 512 && (p.primers = h[9]), !s && d[0] & 67108864 && ((s = !0), (p.selectedtxl = h[26]), Wt(() => (s = !1))), t.$set(p));
      },
      i(h) {
        i || (ve(t.$$.fragment, h), (i = !0));
      },
      o(h) {
        (be(t.$$.fragment, h), (i = !1));
      },
      d(h) {
        (h && m(e), Fe(t), h && m(n), h && m(a));
      },
    }
  );
}
function Tc(l) {
  let e, t, s, n;
  function a(u) {
    l[192](u);
  }
  let i = { primers: l[25] };
  return (
    l[26] !== void 0 && (i.selectedtxl = l[26]),
    (t = new va({ props: i })),
    dt.push(() => zt(t, "selectedtxl", a)),
    {
      c() {
        ((e = c("div")), Re(t.$$.fragment), r(e, "id", "nbprimertablecontainer"), r(e, "class", "svelte-1u6ecyq"));
      },
      m(u, f) {
        (g(u, e, f), Ge(t, e, null), (n = !0));
      },
      p(u, f) {
        const h = {};
        (f[0] & 33554432 && (h.primers = u[25]),
          !s && f[0] & 67108864 && ((s = !0), (h.selectedtxl = u[26]), Wt(() => (s = !1))),
          t.$set(h));
      },
      i(u) {
        n || (ve(t.$$.fragment, u), (n = !0));
      },
      o(u) {
        (be(t.$$.fragment, u), (n = !1));
      },
      d(u) {
        (u && m(e), Fe(t));
      },
    }
  );
}
function Sc(l) {
  let e,
    t,
    s,
    n,
    a,
    i,
    u,
    f,
    h,
    d,
    p,
    v,
    _,
    b,
    w,
    y,
    A,
    q,
    T,
    N,
    M,
    k,
    E,
    P,
    G,
    O,
    K,
    D,
    L,
    R,
    Q,
    ae,
    W,
    x,
    H,
    U,
    le,
    J,
    ie,
    z,
    te,
    pe,
    Y,
    ue,
    he,
    de,
    _e,
    ne,
    $,
    fe,
    we,
    j,
    F,
    ee,
    V,
    me,
    _t,
    Je,
    bt,
    ut,
    it,
    Be,
    We,
    Ie,
    Ce,
    De,
    at,
    Ve,
    $t,
    rt,
    Xt,
    Xe,
    Oe,
    He,
    Ke,
    ke,
    Ue,
    nl,
    At,
    Gt,
    pt,
    Ee,
    Qe,
    ht,
    tt,
    pl,
    Kl,
    Mt,
    hl,
    Et,
    bs,
    kl,
    qs,
    Bt,
    ws,
    Vt,
    Ft,
    Yl,
    Ol,
    Wl,
    ml,
    jl,
    Gl,
    Nt,
    zl,
    rl,
    rs,
    el,
    os,
    tl,
    qt,
    gl,
    Fl,
    Xl,
    Ze,
    vl,
    Ql,
    xl,
    lt,
    st,
    Ml,
    Kt,
    ys,
    Jl,
    as,
    Pt,
    Ht,
    Zl,
    cs,
    Qt,
    As,
    _l,
    Rt,
    Lt,
    Bl,
    bl,
    Ut,
    il,
    ll,
    ql,
    Hl,
    Ul,
    us,
    It,
    Ks,
    zs,
    ol,
    Gs,
    xt,
    tn,
    Fs,
    wl,
    ln,
    $l,
    fs,
    Rs,
    es,
    Cs,
    ds,
    yl,
    un,
    Vl,
    Ys,
    ts,
    Ts,
    Is,
    ps,
    Al,
    Xs,
    sl,
    al,
    Ss,
    Cl,
    ks,
    Ds,
    hs,
    Qs,
    ms,
    xs,
    Ms,
    sn;
  ((u = new Rl({ props: { topic: "Input Sequence" } })), u.$on("click", l[99]));
  function Os(B) {
    l[100](B);
  }
  let js = {};
  (l[1] !== void 0 && (js.seqrec = l[1]), (p = new fa({ props: js })), dt.push(() => zt(p, "seqrec", Os)));
  let Yt = l[33] && l[0] && l[0].length > 1 && or(),
    cl = l[34] && l[0] && l[0].length > 1 && ar(l);
  ((M = new Rl({ props: { topic: "Input Mode" } })), M.$on("click", l[101]));
  function ls(B) {
    l[102](B);
  }
  let nn = { title: "Input Sequence ", data: {}, $$slots: { default: [Ba] }, $$scope: { ctx: l } };
  (l[51] !== void 0 && (nn.dialog = l[51]), (G = new Pl({ props: nn })), dt.push(() => zt(G, "dialog", ls)));
  function gs(B) {
    l[103](B);
  }
  let rn = { title: "Input Mode ", data: {}, $$slots: { default: [Ha] }, $$scope: { ctx: l } };
  (l[52] !== void 0 && (rn.dialog = l[52]), (D = new Pl({ props: rn })), dt.push(() => zt(D, "dialog", gs)));
  function Bs(B) {
    l[104](B);
  }
  let Hs = { title: "Legacy Input Help ", data: {}, $$slots: { default: [Ua] }, $$scope: { ctx: l } };
  (l[48] !== void 0 && (Hs.dialog = l[48]), (Q = new Pl({ props: Hs })), dt.push(() => zt(Q, "dialog", Bs)));
  function Js(B) {
    l[105](B);
  }
  let fn = { title: "InDel Input Help ", data: {}, $$slots: { default: [Va] }, $$scope: { ctx: l } };
  (l[38] !== void 0 && (fn.dialog = l[38]), (x = new Pl({ props: fn })), dt.push(() => zt(x, "dialog", Js)));
  function Zs(B) {
    l[106](B);
  }
  let $s = { title: "AA Point Mutation Input Help ", data: {}, $$slots: { default: [Ka] }, $$scope: { ctx: l } };
  (l[37] !== void 0 && ($s.dialog = l[37]), (le = new Pl({ props: $s })), dt.push(() => zt(le, "dialog", Zs)));
  function ul(B) {
    l[107](B);
  }
  let Tl = { title: "Batch Input Help ", data: {}, $$slots: { default: [Ya] }, $$scope: { ctx: l } };
  (l[36] !== void 0 && (Tl.dialog = l[36]), (z = new Pl({ props: Tl })), dt.push(() => zt(z, "dialog", ul)));
  function El(B) {
    l[108](B);
  }
  let Ae = { title: "Allowed Batch Input ", data: {}, $$slots: { default: [Wa] }, $$scope: { ctx: l } };
  (l[50] !== void 0 && (Ae.dialog = l[50]), (Y = new Pl({ props: Ae })), dt.push(() => zt(Y, "dialog", El)));
  function ot(B) {
    l[109](B);
  }
  let on = { title: "Confine mutations to primer tails", data: {}, $$slots: { default: [za] }, $$scope: { ctx: l } };
  (l[39] !== void 0 && (on.dialog = l[39]), (de = new Pl({ props: on })), dt.push(() => zt(de, "dialog", ot)));
  function ii(B) {
    l[110](B);
  }
  let On = { title: "Preserve degenerate bases", data: {}, $$slots: { default: [Xa] }, $$scope: { ctx: l } };
  (l[40] !== void 0 && (On.dialog = l[40]), ($ = new Pl({ props: On })), dt.push(() => zt($, "dialog", ii)));
  function ri(B) {
    l[111](B);
  }
  let jn = { title: "Merge multiple point mutations", data: {}, $$slots: { default: [Qa] }, $$scope: { ctx: l } };
  (l[47] !== void 0 && (jn.dialog = l[47]), (j = new Pl({ props: jn })), dt.push(() => zt(j, "dialog", ri)));
  function oi(B) {
    l[112](B);
  }
  let Bn = { title: "Codon selection: codon usage", data: {}, $$slots: { default: [xa] }, $$scope: { ctx: l } };
  (l[41] !== void 0 && (Bn.dialog = l[41]), (V = new Pl({ props: Bn })), dt.push(() => zt(V, "dialog", oi)));
  function ai(B) {
    l[113](B);
  }
  let Hn = { title: "Codon selection: parsimony", data: {}, $$slots: { default: [Ja] }, $$scope: { ctx: l } };
  (l[42] !== void 0 && (Hn.dialog = l[42]), (Je = new Pl({ props: Hn })), dt.push(() => zt(Je, "dialog", ai)));
  function ci(B) {
    l[114](B);
  }
  let Un = { title: "Generate NEBuilder HiFi primers", data: {}, $$slots: { default: [Za] }, $$scope: { ctx: l } };
  (l[43] !== void 0 && (Un.dialog = l[43]), (it = new Pl({ props: Un })), dt.push(() => zt(it, "dialog", ci)));
  function ui(B) {
    l[115](B);
  }
  let Vn = { title: "Mutation One-Letter-Codes", data: {}, $$slots: { default: [$a] }, $$scope: { ctx: l } };
  (l[44] !== void 0 && (Vn.dialog = l[44]), (Ie = new Pl({ props: Vn })), dt.push(() => zt(Ie, "dialog", ui)));
  function fi(B) {
    l[116](B);
  }
  let Kn = { title: "Minimum Primer Length", data: {}, $$slots: { default: [ec] }, $$scope: { ctx: l } };
  (l[46] !== void 0 && (Kn.dialog = l[46]), (at = new Pl({ props: Kn })), dt.push(() => zt(at, "dialog", fi)));
  function di(B) {
    l[117](B);
  }
  let Yn = { title: "Minimum Primer Tm", data: {}, $$slots: { default: [tc] }, $$scope: { ctx: l } };
  (l[45] !== void 0 && (Yn.dialog = l[45]), (rt = new Pl({ props: Yn })), dt.push(() => zt(rt, "dialog", di)));
  function pi(B) {
    l[118](B);
  }
  let Wn = { title: "Results", data: {}, $$slots: { default: [lc] }, $$scope: { ctx: l } };
  (l[53] !== void 0 && (Wn.dialog = l[53]), (Oe = new Pl({ props: Wn })), dt.push(() => zt(Oe, "dialog", pi)));
  function hi(B) {
    l[122](B);
  }
  function mi(B) {
    l[123](B);
  }
  let Tn = { title: "Select AA substitution", canClose: !1, $$slots: { default: [ic] }, $$scope: { ctx: l } };
  (l[49] !== void 0 && (Tn.dialog = l[49]),
    l[55] !== void 0 && (Tn.data = l[55]),
    (ke = new Pl({ props: Tn })),
    dt.push(() => zt(ke, "dialog", hi)),
    dt.push(() => zt(ke, "data", mi)),
    ke.$on("close", l[124]));
  function gi(B) {
    l[125](B);
  }
  let zn = { title: "Use NEBuilder Calculator", data: {}, $$slots: { default: [rc] }, $$scope: { ctx: l } };
  (l[54] !== void 0 && (zn.dialog = l[54]),
    (Gt = new Pl({ props: zn })),
    dt.push(() => zt(Gt, "dialog", gi)),
    (Bt = new Rl({ props: { topic: "AA Point Mutation Input" } })),
    Bt.$on("click", l[129]),
    (ml = new Rl({ props: { topic: "InDel Input" } })),
    ml.$on("click", l[132]),
    (el = new Rl({ props: { topic: "Batch Input" } })),
    el.$on("click", l[135]),
    (Ze = new Rl({ props: { topic: "Legacy Input" } })),
    Ze.$on("click", l[138]));
  const Xn = [uc, cc, ac, oc],
    dn = [];
  function Qn(B, ye) {
    return B[12] == "legacy" ? 0 : B[12] == "general" ? 1 : B[12] == "codon" ? 2 : B[12] == "batch" ? 3 : -1;
  }
  ~(Pt = Qn(l)) && (Ht = dn[Pt] = Xn[Pt](l));
  let Nl = !l[18] && Er(l);
  ((ll = new Rl({ props: { topic: "Minimum Primer Length" } })), ll.$on("click", l[182]));
  let Es = l[20] < 15 && Lr();
  ((xt = new Rl({ props: { topic: "Minimum Primer Tm" } })), xt.$on("click", l[184]));
  let Ns = l[21] < 55 && Gr(),
    Ps = l[21] > 72 && Fr();
  ((Ts = new Rl({ props: { topic: "Results" } })), Ts.$on("click", l[185]));
  let Ls = (!l[0] || l[0].length < 1) && Rr();
  const xn = [wc, qc],
    pn = [];
  function Jn(B, ye) {
    return B[9] && B[9].length > 0 ? 0 : 1;
  }
  return (
    (sl = Jn(l)),
    (al = pn[sl] = xn[sl](l)),
    (xs = Ti(l[127][1])),
    {
      c() {
        ((e = c("div")),
          (t = c("div")),
          (t.innerHTML = `<div class="col svelte-1u6ecyq"><div class="description svelte-1u6ecyq">NEBaseChanger assists in the design of primers for site-directed mutagenesis experiments using the Q5 Site-Directed Mutagenesis Kit. 
        		Additional resources for NEBaseChanger, including related products, videos, tutorials, and application notes are available 
        		<a href="#/resources" class="svelte-1u6ecyq">here</a>.\xA0\xA0</div> 
      		<br class="svelte-1u6ecyq"/> 
      		<div class="description svelte-1u6ecyq"><hr class="orange-line svelte-1u6ecyq"/> 
      			<ul class="svelte-1u6ecyq"><li class="svelte-1u6ecyq">Need help <b class="svelte-1u6ecyq">getting started</b>? Please check the <a href="#/help" class="svelte-1u6ecyq">Help</a> page and this 
      					<a href="https://www.neb.com/en-us/tools-and-resources/video-library/nebasechanger-primer-design-tool-tutorial" target="_blank" class="svelte-1u6ecyq">video tutorial</a>.</li> 
      				<li class="svelte-1u6ecyq">Need help <b class="svelte-1u6ecyq">troubleshooting</b> your site-directed mutagenesis experiment? Try the new <a href="#/troubleshoot" class="svelte-1u6ecyq">Troubleshooting Module</a></li></ul> 
      			<hr class="orange-line svelte-1u6ecyq"/></div></div>`),
          (s = S()),
          (n = c("div")),
          (a = c("h2")),
          (i = C("1. Enter new sequence. ")),
          Re(u.$$.fragment),
          (f = S()),
          (h = c("div")),
          (d = c("div")),
          Re(p.$$.fragment),
          (_ = S()),
          (b = c("br")),
          (w = S()),
          Yt && Yt.c(),
          (y = S()),
          cl && cl.c(),
          (A = S()),
          (q = c("div")),
          (T = c("h2")),
          (N = C("2. Select input mode and set mutations. ")),
          Re(M.$$.fragment),
          (k = S()),
          (E = c("div")),
          (P = c("div")),
          Re(G.$$.fragment),
          (K = S()),
          Re(D.$$.fragment),
          (R = S()),
          Re(Q.$$.fragment),
          (W = S()),
          Re(x.$$.fragment),
          (U = S()),
          Re(le.$$.fragment),
          (ie = S()),
          Re(z.$$.fragment),
          (pe = S()),
          Re(Y.$$.fragment),
          (he = S()),
          Re(de.$$.fragment),
          (ne = S()),
          Re($.$$.fragment),
          (we = S()),
          Re(j.$$.fragment),
          (ee = S()),
          Re(V.$$.fragment),
          (_t = S()),
          Re(Je.$$.fragment),
          (ut = S()),
          Re(it.$$.fragment),
          (We = S()),
          Re(Ie.$$.fragment),
          (De = S()),
          Re(at.$$.fragment),
          ($t = S()),
          Re(rt.$$.fragment),
          (Xe = S()),
          Re(Oe.$$.fragment),
          (Ke = S()),
          Re(ke.$$.fragment),
          (At = S()),
          Re(Gt.$$.fragment),
          (Ee = S()),
          (Qe = c("div")),
          (ht = c("div")),
          (tt = c("div")),
          (pl = c("div")),
          (Kl = S()),
          (Mt = c("div")),
          (hl = c("span")),
          (Et = c("input")),
          (bs = S()),
          (kl = c("label")),
          (kl.textContent = "AA Point Mutation"),
          (qs = S()),
          Re(Bt.$$.fragment),
          (ws = S()),
          (Vt = c("span")),
          (Ft = c("input")),
          (Yl = S()),
          (Ol = c("label")),
          (Ol.textContent = "Indel/Substitution"),
          (Wl = S()),
          Re(ml.$$.fragment),
          (jl = S()),
          (Gl = c("span")),
          (Nt = c("input")),
          (zl = S()),
          (rl = c("label")),
          (rl.textContent = "Batch (mixed)"),
          (rs = S()),
          Re(el.$$.fragment),
          (os = S()),
          (tl = c("span")),
          (qt = c("input")),
          (gl = S()),
          (Fl = c("label")),
          (Fl.textContent = "Legacy (emulate V1)"),
          (Xl = S()),
          Re(Ze.$$.fragment),
          (vl = S()),
          (Ql = c("br")),
          (xl = S()),
          (lt = c("div")),
          (st = c("label")),
          (st.textContent = "Primer Set ID:"),
          (Ml = S()),
          (Kt = c("input")),
          (ys = S()),
          (Jl = c("br")),
          (as = S()),
          Ht && Ht.c(),
          (Zl = S()),
          Nl && Nl.c(),
          (cs = S()),
          (Qt = c("br")),
          (As = S()),
          (_l = c("div")),
          (Rt = c("div")),
          (Lt = c("div")),
          (Bl = c("label")),
          (Bl.textContent = "Minimum Primer Length (nt):"),
          (bl = S()),
          (Ut = c("input")),
          (il = C(`
	        \xA0\xA0
		   	`)),
          Re(ll.$$.fragment),
          (ql = S()),
          Es && Es.c(),
          (Hl = C(`
	    \xA0\xA0`)),
          (Ul = c("br")),
          (us = S()),
          (It = c("div")),
          (Ks = c("label")),
          (Ks.textContent = "Minimum Primer Tm (\xB0C):"),
          (zs = S()),
          (ol = c("input")),
          (Gs = C(`
	        \xA0\xA0
		   	`)),
          Re(xt.$$.fragment),
          (tn = S()),
          Ns && Ns.c(),
          (Fs = S()),
          Ps && Ps.c(),
          (wl = S()),
          (ln = c("br")),
          ($l = c("br")),
          (fs = S()),
          (Rs = c("div")),
          (es = c("div")),
          (Cs = c("button")),
          (Cs.textContent = "Build Primers"),
          (ds = S()),
          (yl = c("button")),
          (yl.textContent = "Clear Primers"),
          (un = S()),
          (Vl = c("div")),
          (Ys = c("h2")),
          (ts = C("3. Results ")),
          Re(Ts.$$.fragment),
          (Is = S()),
          (ps = c("div")),
          (Al = c("div")),
          Ls && Ls.c(),
          (Xs = S()),
          al.c(),
          (Ss = S()),
          (Cl = c("br")),
          (ks = S()),
          (Ds = c("br")),
          (hs = S()),
          (Qs = c("div")),
          (Qs.innerHTML = '<div class="col svelte-1u6ecyq"><hr class="svelte-1u6ecyq"/></div>'),
          r(t, "class", "row svelte-1u6ecyq"),
          r(a, "class", "svelte-1u6ecyq"),
          r(b, "class", "svelte-1u6ecyq"),
          r(d, "class", "col svelte-1u6ecyq"),
          r(h, "class", "row svelte-1u6ecyq"),
          r(n, "class", "appsection svelte-1u6ecyq"),
          r(T, "class", "svelte-1u6ecyq"),
          r(P, "class", "col svelte-1u6ecyq"),
          r(E, "class", "row svelte-1u6ecyq"),
          r(pl, "id", "inputsequence"),
          r(pl, "unselectable", "on"),
          r(pl, "class", "svelte-1u6ecyq"),
          r(Et, "type", "radio"),
          r(Et, "id", "codonin"),
          r(Et, "name", "inputmode"),
          (Et.__value = "codon"),
          (Et.value = Et.__value),
          r(Et, "class", "svelte-1u6ecyq"),
          r(kl, "for", "codonin"),
          r(kl, "class", "svelte-1u6ecyq"),
          r(hl, "class", "svelte-1u6ecyq"),
          r(Ft, "type", "radio"),
          r(Ft, "id", "singlein"),
          r(Ft, "name", "inputmode"),
          (Ft.__value = "general"),
          (Ft.value = Ft.__value),
          r(Ft, "class", "svelte-1u6ecyq"),
          r(Ol, "for", "singlein"),
          r(Ol, "class", "svelte-1u6ecyq"),
          r(Vt, "class", "svelte-1u6ecyq"),
          r(Nt, "type", "radio"),
          r(Nt, "id", "batchin"),
          r(Nt, "name", "inputmode"),
          (Nt.__value = "batch"),
          (Nt.value = Nt.__value),
          r(Nt, "class", "svelte-1u6ecyq"),
          r(rl, "for", "batchin"),
          r(rl, "class", "svelte-1u6ecyq"),
          r(Gl, "class", "svelte-1u6ecyq"),
          r(qt, "type", "radio"),
          r(qt, "id", "legacyin"),
          r(qt, "name", "inputmode"),
          (qt.__value = "legacy"),
          (qt.value = qt.__value),
          r(qt, "class", "svelte-1u6ecyq"),
          r(Fl, "for", "legacyin"),
          r(Fl, "class", "svelte-1u6ecyq"),
          r(tl, "class", "svelte-1u6ecyq"),
          r(Mt, "class", " svelte-1u6ecyq"),
          r(Ql, "class", "svelte-1u6ecyq"),
          r(st, "for", "opid"),
          r(st, "class", "svelte-1u6ecyq"),
          r(Kt, "type", "text"),
          r(Kt, "id", "opid"),
          r(Kt, "placeholder", "ID for primer set"),
          r(Kt, "class", "svelte-1u6ecyq"),
          r(lt, "class", "svelte-1u6ecyq"),
          r(Jl, "class", "svelte-1u6ecyq"),
          r(tt, "id", "design"),
          r(tt, "class", " svelte-1u6ecyq"),
          r(Qt, "class", "svelte-1u6ecyq"),
          r(ht, "class", "col svelte-1u6ecyq"),
          r(Qe, "class", "row svelte-1u6ecyq"),
          r(Bl, "for", "userminlen"),
          r(Bl, "class", "svelte-1u6ecyq"),
          r(Ut, "type", "number"),
          r(Ut, "id", "userminlen"),
          r(Ut, "min", "15"),
          r(Ut, "class", "svelte-1u6ecyq"),
          r(Lt, "class", "range-holder svelte-1u6ecyq"),
          r(Ul, "class", "svelte-1u6ecyq"),
          r(Ks, "for", "usermintm"),
          r(Ks, "class", "svelte-1u6ecyq"),
          r(ol, "type", "number"),
          r(ol, "id", "usermintm"),
          r(ol, "min", "55"),
          r(ol, "max", "72"),
          r(ol, "class", "svelte-1u6ecyq"),
          r(It, "class", "range-holder svelte-1u6ecyq"),
          r(ln, "class", "svelte-1u6ecyq"),
          r($l, "class", "svelte-1u6ecyq"),
          r(Rt, "class", "col svelte-1u6ecyq"),
          r(_l, "class", "row svelte-1u6ecyq"),
          r(Cs, "class", "button btn btn-link-cta svelte-1u6ecyq"),
          r(yl, "class", "button btn btn-link svelte-1u6ecyq"),
          r(es, "class", "col svelte-1u6ecyq"),
          r(Rs, "class", "row svelte-1u6ecyq"),
          r(q, "class", "appsection svelte-1u6ecyq"),
          r(Ys, "class", "svelte-1u6ecyq"),
          r(Cl, "class", "svelte-1u6ecyq"),
          r(Ds, "class", "svelte-1u6ecyq"),
          r(Al, "class", "col svelte-1u6ecyq"),
          r(ps, "class", "row svelte-1u6ecyq"),
          r(Vl, "class", "appsection svelte-1u6ecyq"),
          r(Qs, "class", "row svelte-1u6ecyq"),
          r(e, "class", "container svelte-1u6ecyq"),
          xs.p(Et, Ft, Nt, qt));
      },
      m(B, ye) {
        (g(B, e, ye),
          o(e, t),
          o(e, s),
          o(e, n),
          o(n, a),
          o(a, i),
          Ge(u, a, null),
          o(n, f),
          o(n, h),
          o(h, d),
          Ge(p, d, null),
          o(d, _),
          o(d, b),
          o(d, w),
          Yt && Yt.m(d, null),
          o(d, y),
          cl && cl.m(d, null),
          o(e, A),
          o(e, q),
          o(q, T),
          o(T, N),
          Ge(M, T, null),
          o(q, k),
          o(q, E),
          o(E, P),
          Ge(G, P, null),
          o(P, K),
          Ge(D, P, null),
          o(P, R),
          Ge(Q, P, null),
          o(P, W),
          Ge(x, P, null),
          o(P, U),
          Ge(le, P, null),
          o(P, ie),
          Ge(z, P, null),
          o(P, pe),
          Ge(Y, P, null),
          o(P, he),
          Ge(de, P, null),
          o(P, ne),
          Ge($, P, null),
          o(P, we),
          Ge(j, P, null),
          o(P, ee),
          Ge(V, P, null),
          o(P, _t),
          Ge(Je, P, null),
          o(P, ut),
          Ge(it, P, null),
          o(P, We),
          Ge(Ie, P, null),
          o(P, De),
          Ge(at, P, null),
          o(P, $t),
          Ge(rt, P, null),
          o(P, Xe),
          Ge(Oe, P, null),
          o(P, Ke),
          Ge(ke, P, null),
          o(P, At),
          Ge(Gt, P, null),
          o(q, Ee),
          o(q, Qe),
          o(Qe, ht),
          o(ht, tt),
          o(tt, pl),
          o(tt, Kl),
          o(tt, Mt),
          o(Mt, hl),
          o(hl, Et),
          (Et.checked = Et.__value === l[12]),
          o(hl, bs),
          o(hl, kl),
          o(Mt, qs),
          Ge(Bt, Mt, null),
          o(Mt, ws),
          o(Mt, Vt),
          o(Vt, Ft),
          (Ft.checked = Ft.__value === l[12]),
          o(Vt, Yl),
          o(Vt, Ol),
          o(Mt, Wl),
          Ge(ml, Mt, null),
          o(Mt, jl),
          o(Mt, Gl),
          o(Gl, Nt),
          (Nt.checked = Nt.__value === l[12]),
          o(Gl, zl),
          o(Gl, rl),
          o(Mt, rs),
          Ge(el, Mt, null),
          o(Mt, os),
          o(Mt, tl),
          o(tl, qt),
          (qt.checked = qt.__value === l[12]),
          o(tl, gl),
          o(tl, Fl),
          o(Mt, Xl),
          Ge(Ze, Mt, null),
          o(tt, vl),
          o(tt, Ql),
          o(tt, xl),
          o(tt, lt),
          o(lt, st),
          o(lt, Ml),
          o(lt, Kt),
          nt(Kt, l[11]),
          o(tt, ys),
          o(tt, Jl),
          o(ht, as),
          ~Pt && dn[Pt].m(ht, null),
          o(ht, Zl),
          Nl && Nl.m(ht, null),
          o(ht, cs),
          o(ht, Qt),
          o(q, As),
          o(q, _l),
          o(_l, Rt),
          o(Rt, Lt),
          o(Lt, Bl),
          o(Lt, bl),
          o(Lt, Ut),
          nt(Ut, l[20]),
          o(Lt, il),
          Ge(ll, Lt, null),
          o(Lt, ql),
          Es && Es.m(Lt, null),
          o(Rt, Hl),
          o(Rt, Ul),
          o(Rt, us),
          o(Rt, It),
          o(It, Ks),
          o(It, zs),
          o(It, ol),
          nt(ol, l[21]),
          o(It, Gs),
          Ge(xt, It, null),
          o(It, tn),
          Ns && Ns.m(It, null),
          o(It, Fs),
          Ps && Ps.m(It, null),
          o(Rt, wl),
          o(Rt, ln),
          o(Rt, $l),
          o(q, fs),
          o(q, Rs),
          o(Rs, es),
          o(es, Cs),
          o(es, ds),
          o(es, yl),
          o(e, un),
          o(e, Vl),
          o(Vl, Ys),
          o(Ys, ts),
          Ge(Ts, Ys, null),
          o(Vl, Is),
          o(Vl, ps),
          o(ps, Al),
          Ls && Ls.m(Al, null),
          o(Al, Xs),
          pn[sl].m(Al, null),
          o(Al, Ss),
          o(Al, Cl),
          o(Al, ks),
          o(Al, Ds),
          o(e, hs),
          o(e, Qs),
          (ms = !0),
          Ms ||
            ((sn = [
              ce(Et, "change", l[126]),
              ce(Et, "change", l[128]),
              ce(Ft, "change", l[130]),
              ce(Ft, "change", l[131]),
              ce(Nt, "change", l[133]),
              ce(Nt, "change", l[134]),
              ce(qt, "change", l[136]),
              ce(qt, "change", l[137]),
              ce(Kt, "input", l[139]),
              ce(Ut, "input", l[180]),
              ce(Ut, "input", l[181]),
              ce(ol, "input", l[183]),
              ce(ol, "input", l[75]),
              ce(Cs, "click", l[78]),
              ce(yl, "click", l[79]),
            ]),
            (Ms = !0)));
      },
      p(B, ye) {
        const Zn = {};
        (!v && ye[0] & 2 && ((v = !0), (Zn.seqrec = B[1]), Wt(() => (v = !1))),
          p.$set(Zn),
          B[33] && B[0] && B[0].length > 1 ? Yt || ((Yt = or()), Yt.c(), Yt.m(d, y)) : Yt && (Yt.d(1), (Yt = null)),
          B[34] && B[0] && B[0].length > 1 ? (cl ? cl.p(B, ye) : ((cl = ar(B)), cl.c(), cl.m(d, null))) : cl && (cl.d(1), (cl = null)));
        const Sn = {};
        (ye[8] & 4194304 && (Sn.$$scope = { dirty: ye, ctx: B }),
          !O && ye[1] & 1048576 && ((O = !0), (Sn.dialog = B[51]), Wt(() => (O = !1))),
          G.$set(Sn));
        const kn = {};
        (ye[8] & 4194304 && (kn.$$scope = { dirty: ye, ctx: B }),
          !L && ye[1] & 2097152 && ((L = !0), (kn.dialog = B[52]), Wt(() => (L = !1))),
          D.$set(kn));
        const Mn = {};
        (ye[8] & 4194304 && (Mn.$$scope = { dirty: ye, ctx: B }),
          !ae && ye[1] & 131072 && ((ae = !0), (Mn.dialog = B[48]), Wt(() => (ae = !1))),
          Q.$set(Mn));
        const En = {};
        (ye[8] & 4194304 && (En.$$scope = { dirty: ye, ctx: B }),
          !H && ye[1] & 128 && ((H = !0), (En.dialog = B[38]), Wt(() => (H = !1))),
          x.$set(En));
        const Nn = {};
        (ye[8] & 4194304 && (Nn.$$scope = { dirty: ye, ctx: B }),
          !J && ye[1] & 64 && ((J = !0), (Nn.dialog = B[37]), Wt(() => (J = !1))),
          le.$set(Nn));
        const Pn = {};
        (ye[8] & 4194304 && (Pn.$$scope = { dirty: ye, ctx: B }),
          !te && ye[1] & 32 && ((te = !0), (Pn.dialog = B[36]), Wt(() => (te = !1))),
          z.$set(Pn));
        const I = {};
        (ye[8] & 4194304 && (I.$$scope = { dirty: ye, ctx: B }),
          !ue && ye[1] & 524288 && ((ue = !0), (I.dialog = B[50]), Wt(() => (ue = !1))),
          Y.$set(I));
        const X = {};
        (ye[8] & 4194304 && (X.$$scope = { dirty: ye, ctx: B }),
          !_e && ye[1] & 256 && ((_e = !0), (X.dialog = B[39]), Wt(() => (_e = !1))),
          de.$set(X));
        const re = {};
        (ye[8] & 4194304 && (re.$$scope = { dirty: ye, ctx: B }),
          !fe && ye[1] & 512 && ((fe = !0), (re.dialog = B[40]), Wt(() => (fe = !1))),
          $.$set(re));
        const se = {};
        (ye[8] & 4194304 && (se.$$scope = { dirty: ye, ctx: B }),
          !F && ye[1] & 65536 && ((F = !0), (se.dialog = B[47]), Wt(() => (F = !1))),
          j.$set(se));
        const Se = {};
        (ye[8] & 4194304 && (Se.$$scope = { dirty: ye, ctx: B }),
          !me && ye[1] & 1024 && ((me = !0), (Se.dialog = B[41]), Wt(() => (me = !1))),
          V.$set(Se));
        const Me = {};
        (ye[8] & 4194304 && (Me.$$scope = { dirty: ye, ctx: B }),
          !bt && ye[1] & 2048 && ((bt = !0), (Me.dialog = B[42]), Wt(() => (bt = !1))),
          Je.$set(Me));
        const Le = {};
        (ye[8] & 4194304 && (Le.$$scope = { dirty: ye, ctx: B }),
          !Be && ye[1] & 4096 && ((Be = !0), (Le.dialog = B[43]), Wt(() => (Be = !1))),
          it.$set(Le));
        const je = {};
        (ye[8] & 4194304 && (je.$$scope = { dirty: ye, ctx: B }),
          !Ce && ye[1] & 8192 && ((Ce = !0), (je.dialog = B[44]), Wt(() => (Ce = !1))),
          Ie.$set(je));
        const oe = {};
        (ye[8] & 4194304 && (oe.$$scope = { dirty: ye, ctx: B }),
          !Ve && ye[1] & 32768 && ((Ve = !0), (oe.dialog = B[46]), Wt(() => (Ve = !1))),
          at.$set(oe));
        const ge = {};
        (ye[8] & 4194304 && (ge.$$scope = { dirty: ye, ctx: B }),
          !Xt && ye[1] & 16384 && ((Xt = !0), (ge.dialog = B[45]), Wt(() => (Xt = !1))),
          rt.$set(ge));
        const xe = {};
        (ye[8] & 4194304 && (xe.$$scope = { dirty: ye, ctx: B }),
          !He && ye[1] & 4194304 && ((He = !0), (xe.dialog = B[53]), Wt(() => (He = !1))),
          Oe.$set(xe));
        const et = {};
        ((ye[0] & 64) | (ye[1] & 17039360) | (ye[8] & 4194304) && (et.$$scope = { dirty: ye, ctx: B }),
          !Ue && ye[1] & 262144 && ((Ue = !0), (et.dialog = B[49]), Wt(() => (Ue = !1))),
          !nl && ye[1] & 16777216 && ((nl = !0), (et.data = B[55]), Wt(() => (nl = !1))),
          ke.$set(et));
        const Ct = {};
        (ye[8] & 4194304 && (Ct.$$scope = { dirty: ye, ctx: B }),
          !pt && ye[1] & 8388608 && ((pt = !0), (Ct.dialog = B[54]), Wt(() => (pt = !1))),
          Gt.$set(Ct),
          ye[0] & 4096 && (Et.checked = Et.__value === B[12]),
          ye[0] & 4096 && (Ft.checked = Ft.__value === B[12]),
          ye[0] & 4096 && (Nt.checked = Nt.__value === B[12]),
          ye[0] & 4096 && (qt.checked = qt.__value === B[12]),
          ye[0] & 2048 && Kt.value !== B[11] && nt(Kt, B[11]));
        let Tt = Pt;
        ((Pt = Qn(B)),
          Pt === Tt
            ? ~Pt && dn[Pt].p(B, ye)
            : (Ht &&
                (vs(),
                be(dn[Tt], 1, 1, () => {
                  dn[Tt] = null;
                }),
                _s()),
              ~Pt ? ((Ht = dn[Pt]), Ht ? Ht.p(B, ye) : ((Ht = dn[Pt] = Xn[Pt](B)), Ht.c()), ve(Ht, 1), Ht.m(ht, Zl)) : (Ht = null)),
          B[18]
            ? Nl &&
              (vs(),
              be(Nl, 1, 1, () => {
                Nl = null;
              }),
              _s())
            : Nl
              ? (Nl.p(B, ye), ye[0] & 262144 && ve(Nl, 1))
              : ((Nl = Er(B)), Nl.c(), ve(Nl, 1), Nl.m(ht, cs)),
          ye[0] & 1048576 && Us(Ut.value) !== B[20] && nt(Ut, B[20]),
          B[20] < 15 ? Es || ((Es = Lr()), Es.c(), Es.m(Lt, null)) : Es && (Es.d(1), (Es = null)),
          ye[0] & 2097152 && Us(ol.value) !== B[21] && nt(ol, B[21]),
          B[21] < 55 ? Ns || ((Ns = Gr()), Ns.c(), Ns.m(It, Fs)) : Ns && (Ns.d(1), (Ns = null)),
          B[21] > 72 ? Ps || ((Ps = Fr()), Ps.c(), Ps.m(It, null)) : Ps && (Ps.d(1), (Ps = null)),
          !B[0] || B[0].length < 1 ? Ls || ((Ls = Rr()), Ls.c(), Ls.m(Al, Xs)) : Ls && (Ls.d(1), (Ls = null)));
        let wt = sl;
        ((sl = Jn(B)),
          sl === wt
            ? pn[sl].p(B, ye)
            : (vs(),
              be(pn[wt], 1, 1, () => {
                pn[wt] = null;
              }),
              _s(),
              (al = pn[sl]),
              al ? al.p(B, ye) : ((al = pn[sl] = xn[sl](B)), al.c()),
              ve(al, 1),
              al.m(Al, Ss)));
      },
      i(B) {
        ms ||
          (ve(u.$$.fragment, B),
          ve(p.$$.fragment, B),
          ve(M.$$.fragment, B),
          ve(G.$$.fragment, B),
          ve(D.$$.fragment, B),
          ve(Q.$$.fragment, B),
          ve(x.$$.fragment, B),
          ve(le.$$.fragment, B),
          ve(z.$$.fragment, B),
          ve(Y.$$.fragment, B),
          ve(de.$$.fragment, B),
          ve($.$$.fragment, B),
          ve(j.$$.fragment, B),
          ve(V.$$.fragment, B),
          ve(Je.$$.fragment, B),
          ve(it.$$.fragment, B),
          ve(Ie.$$.fragment, B),
          ve(at.$$.fragment, B),
          ve(rt.$$.fragment, B),
          ve(Oe.$$.fragment, B),
          ve(ke.$$.fragment, B),
          ve(Gt.$$.fragment, B),
          ve(Bt.$$.fragment, B),
          ve(ml.$$.fragment, B),
          ve(el.$$.fragment, B),
          ve(Ze.$$.fragment, B),
          ve(Ht),
          ve(Nl),
          ve(ll.$$.fragment, B),
          ve(xt.$$.fragment, B),
          ve(Ts.$$.fragment, B),
          ve(al),
          (ms = !0));
      },
      o(B) {
        (be(u.$$.fragment, B),
          be(p.$$.fragment, B),
          be(M.$$.fragment, B),
          be(G.$$.fragment, B),
          be(D.$$.fragment, B),
          be(Q.$$.fragment, B),
          be(x.$$.fragment, B),
          be(le.$$.fragment, B),
          be(z.$$.fragment, B),
          be(Y.$$.fragment, B),
          be(de.$$.fragment, B),
          be($.$$.fragment, B),
          be(j.$$.fragment, B),
          be(V.$$.fragment, B),
          be(Je.$$.fragment, B),
          be(it.$$.fragment, B),
          be(Ie.$$.fragment, B),
          be(at.$$.fragment, B),
          be(rt.$$.fragment, B),
          be(Oe.$$.fragment, B),
          be(ke.$$.fragment, B),
          be(Gt.$$.fragment, B),
          be(Bt.$$.fragment, B),
          be(ml.$$.fragment, B),
          be(el.$$.fragment, B),
          be(Ze.$$.fragment, B),
          be(Ht),
          be(Nl),
          be(ll.$$.fragment, B),
          be(xt.$$.fragment, B),
          be(Ts.$$.fragment, B),
          be(al),
          (ms = !1));
      },
      d(B) {
        (B && m(e),
          Fe(u),
          Fe(p),
          Yt && Yt.d(),
          cl && cl.d(),
          Fe(M),
          Fe(G),
          Fe(D),
          Fe(Q),
          Fe(x),
          Fe(le),
          Fe(z),
          Fe(Y),
          Fe(de),
          Fe($),
          Fe(j),
          Fe(V),
          Fe(Je),
          Fe(it),
          Fe(Ie),
          Fe(at),
          Fe(rt),
          Fe(Oe),
          Fe(ke),
          Fe(Gt),
          Fe(Bt),
          Fe(ml),
          Fe(el),
          Fe(Ze),
          ~Pt && dn[Pt].d(),
          Nl && Nl.d(),
          Fe(ll),
          Es && Es.d(),
          Fe(xt),
          Ns && Ns.d(),
          Ps && Ps.d(),
          Fe(Ts),
          Ls && Ls.d(),
          pn[sl].d(),
          xs.r(),
          (Ms = !1),
          Zt(sn));
      },
    }
  );
}
const kc = "multiple",
  Mc = 200,
  Vr = (l, e) => l[1].localeCompare(e[1]);
function Ec(l, e, t) {
  let s, n, a;
  var i = " ",
    u,
    f = {
      seq: "",
      id: "unknown",
      format: "unknown",
      header: "",
      isCircular: !0,
      useRevComp: !1,
      isVector: !0,
      type: "dna",
      len: 0,
      start: 0,
      end: 0,
      frag: null,
    },
    h,
    d,
    p,
    v = "",
    _ = [],
    b,
    w,
    y = "",
    A = !1,
    q = !1,
    T = [],
    N = [],
    M = 0,
    k = !1;
  let E = "sdmprimers.txt",
    P = "sdmprimer.txt",
    G = "sdmfinalseq.fa",
    O = "sdmfinalallseqs.fa",
    K = "sdmsummary.txt";
  var D = "",
    L = "codon",
    R = "",
    Q = 1,
    ae = null,
    W = !1,
    x = "maxcut",
    H = !1,
    U = "",
    le = !1,
    J = [],
    ie = [],
    z = [],
    te = 0,
    pe = !1,
    Y = !1,
    ue = 15,
    he = !1,
    de = function (I) {
      he || t(20, (ue = I));
    },
    _e = 55,
    ne = {
      Substitution: "Start and end coordinates should flank the sequence to be replaced.",
      Insertion: "Start and end coordinates should be consecutive and flank the insertion site.",
      Deletion: "Start and end coordinates should flank the sequence to be deleted.",
    },
    $,
    fe,
    we,
    j,
    F,
    ee,
    V,
    me,
    _t,
    Je,
    bt,
    ut,
    it,
    Be,
    We,
    Ie,
    Ce,
    De,
    at,
    Ve = {},
    $t = function (I) {
      return (
        (I = I.toUpperCase()),
        {
          GCG: ["Ala"],
          GCC: ["Ala"],
          GCA: ["Ala"],
          GCT: ["Ala"],
          CGC: ["Arg"],
          CGT: ["Arg"],
          CGG: ["Arg"],
          CGA: ["Arg"],
          AGA: ["Arg"],
          AGG: ["Arg"],
          AAC: ["Asn"],
          AAT: ["Asn"],
          GAT: ["Asp"],
          GAC: ["Asp"],
          TGC: ["Cys"],
          TGT: ["Cys"],
          CAG: ["Gln"],
          CAA: ["Gln"],
          GAA: ["Glu"],
          GAG: ["Glu"],
          GGC: ["Gly"],
          GGT: ["Gly"],
          GGG: ["Gly"],
          GGA: ["Gly"],
          CAT: ["His"],
          CAC: ["His"],
          ATT: ["Ile"],
          ATC: ["Ile"],
          ATA: ["Ile"],
          CTG: ["Leu"],
          TTG: ["Leu"],
          TTA: ["Leu"],
          CTT: ["Leu"],
          CTC: ["Leu"],
          CTA: ["Leu"],
          AAA: ["Lys"],
          AAG: ["Lys"],
          ATG: ["Met"],
          TTT: ["Phe"],
          TTC: ["Phe"],
          CCG: ["Pro"],
          CCA: ["Pro"],
          CCT: ["Pro"],
          CCC: ["Pro"],
          AGC: ["Ser"],
          AGT: ["Ser"],
          TCG: ["Ser"],
          TCT: ["Ser"],
          TCC: ["Ser"],
          TCA: ["Ser"],
          ACC: ["Thr"],
          ACG: ["Thr"],
          ACT: ["Thr"],
          ACA: ["Thr"],
          TGG: ["Trp"],
          TAT: ["Tyr"],
          TAC: ["Tyr"],
          GTG: ["Val"],
          GTT: ["Val"],
          GTC: ["Val"],
          GTA: ["Val"],
          TAA: ["Ter"],
          TGA: ["Ter"],
          TAG: ["Ter"],
          NNK: ["All"],
        }[I][0] || "NNK"
      );
    },
    rt = function (I) {
      var X = Te.AMINO_ACID_OLC[I],
        re;
      return ((re = Te.cut.Ecoli_K12.filter((se) => se[0] == X)), re.sort((se, Se) => Se[2] - se[2]), re[0][1]);
    },
    Xt = function (I, X) {
      X = X.toUpperCase();
      var re = Te.AMINO_ACID_OLC[I],
        se = [],
        Se = function (Me, Le) {
          var je = 0;
          return ((je += Me[0] == Le[0] ? 0 : 1), (je += Me[1] == Le[1] ? 0 : 1), (je += Me[2] == Le[2] ? 0 : 1), je);
        };
      return ((se = Te.cut.Ecoli_K12.filter((Me) => Me[0] == re)), se.sort((Me, Le) => Se(Me[1], X) - Se(Le[1], X)), se[0][1]);
    },
    Xe = "",
    Oe = "",
    He;
  const Ke = function (I) {
      var re = I.target.result;
      (t(22, (Xe = re)), (I.target.value = ""));
    },
    ke = function (I) {},
    Ue = function (I) {
      const X = new FileReader();
      (X.readAsText(I), X.addEventListener("loadend", Ke), X.addEventListener("error", ke));
    };
  var nl = function () {
      const I = document.getElementById("batchfileinput").files[0];
      I && Ue(I);
    },
    At = function (I) {
      return Jt.buildFasta(I, { seq: i, id: u ? u.id : "", circular: f.isCircular });
    },
    Gt = function () {
      return le && N.length > 0 ? N : T;
    },
    pt = function (I) {
      return { data: Jt.primerList(I), warnings: [] };
    },
    Ee = function (I) {
      return { data: Jt.summary(I), warnings: [] };
    },
    Qe = function (I, X, re) {
      var se;
      if (!X || X.length === 0 || !X[0]) {
        t(32, (z = ["Download: no primer pair is selected."]));
        return;
      }
      try {
        se = I(X);
      } catch (Me) {
        se = { data: "", warnings: ["Download: unexpected error (" + (Me && Me.message ? Me.message : Me) + ")."] };
      }
      if ((t(32, (z = se.warnings)), !se.data)) {
        z.length === 0 && t(32, (z = ["Download: nothing to download."]));
        return;
      }
      var Se = new Blob([se.data], { type: "text/plain;charset=utf-8" });
      Ta(Se, re, !0);
    },
    ht = function () {
      (t(29, (G = Jt.downloadName(G, "sdmfinalseq.fa", ".fa"))), Qe(At, [T[M]], G));
    },
    tt = function () {
      (t(30, (O = Jt.downloadName(O, "sdmfinalallseqs.fa", ".fa"))), Qe(At, T, O));
    },
    pl = function () {
      (t(27, (E = Jt.downloadName(E, "sdmprimers.txt", ".txt"))), Qe(pt, Gt(), E));
    },
    Kl = function () {
      (t(28, (P = Jt.downloadName(P, "sdmprimer.txt", ".txt"))), Qe(pt, [T[M]], P));
    },
    Mt = function () {
      (t(31, (K = Jt.downloadName(K, "sdmsummary.txt", ".txt"))), Qe(Ee, Gt(), K));
    },
    hl = function () {
      let I = document.createElement("a");
      ((I.target = "_blank"), I.setAttribute("id", "dynamicNBPLink"));
      let X = "https://nebuildercalculator.neb.com/?",
        re = [];
      for (let Se = 0; Se < N.length; ++Se) re.push({ name: "frag_" + N[Se].idprefix.replace(/:.../g, ""), bplength: N[Se].fraglen });
      let se = encodeURIComponent(JSON.stringify(re));
      ((I.href = X + se), I.click(), I.remove());
    },
    Et = function () {
      q && (t(7, (A = !0)), t(16, (H = !1)));
    },
    bs = function () {
      k && (t(7, (A = !0)), t(16, (H = !1)));
    };
  let kl;
  const qs = function (I) {
    (clearTimeout(kl), (kl = setTimeout(() => {}, 2e3)));
  };
  var Bt = function (I, X, re, se) {
      var Se = X - 1,
        Me = re - 1;
      return (
        f.isCircular
          ? ((Se = Se % I.length),
            (Me = Me % I.length),
            Se < 0 && (Se += I.length),
            Me < 0 && (Me += I.length),
            se == "replace"
              ? Me > Se
                ? (y = I.slice(Se + 1, Me))
                : Me < Se
                  ? (y = I.slice(Se + 1) + I.slice(0, Me))
                  : (y = "")
              : (y = ""))
          : Se < 0 || Me < 0 || Me <= Se
            ? (y = "")
            : se == "replace"
              ? (y = I.slice(Se + 1, Me))
              : (y = ""),
        y
      );
    },
    ws = function (I, X) {
      if (I == null || I === "") return NaN;
      const re = Number(I);
      return Number.isInteger(re) && re >= 1 && re <= X ? re - 1 : NaN;
    },
    Vt = null,
    Ft = function () {
      (clearTimeout(Vt),
        (Vt = setTimeout(() => {
          ((Vt = null), t(19, (te += 1)));
        }, 250)));
    },
    Yl = function () {
      (clearTimeout(Vt), (Vt = null), t(19, (te += 1)));
    },
    Ol = function () {
      (t(9, (T = [])), t(95, (J = [])), t(32, (z = [])), t(26, (M = 0)));
    },
    Wl = function (I, X) {
      return (I || []).filter(function (re) {
        var se = ct.verifyPair(re, i, f.isCircular);
        return (
          se && X.push("internal check failed for " + re.idprefix + ": " + se + " - primers not shown. Please report this to NEB."),
          !se
        );
      });
    },
    ml = function () {
      return ae !== null && Number(ae) !== Number(Q);
    },
    jl = function (I, X, re, se) {
      return (
        I +
        " skipped: the ORF start moved from " +
        ae +
        " to " +
        Q +
        ", and codon " +
        se +
        " is now " +
        re +
        ", not " +
        X +
        ". Edit or remove the op, or move the ORF start back to " +
        ae +
        "."
      );
    },
    Gl = function (I) {
      if (W) {
        W = !1;
        return;
      }
      ae = Q;
    },
    Nt = function () {
      var I = [],
        X = [],
        re = function () {
          return { primers: I, warnings: X };
        },
        se,
        Se = [],
        Me = !f.isCircular;
      if (i.length < 60) return (t(33, (pe = !0)), re());
      if ((t(33, (pe = !1)), jt.sequenceLengthError(i.length))) return (t(34, (Y = !0)), re());
      if ((t(34, (Y = !1)), L == "general" || L == "legacy")) {
        t(59, (_l = !0));
        let Le = jt.replacementError(v, q === !0);
        if (Le) return (X.push(Le), re());
        let je = jt.stripReplacement(v);
        if (d == null || p == null) return ((h = "not_ready"), re());
        if (f.isCircular && (jt.positionError("start position", d, i.length) || jt.positionError("end position", p, i.length)))
          return (X.push(jt.positionError("start position", d, i.length) || jt.positionError("end position", p, i.length)), re());
        {
          let oe = ct.classifyOp(d, p, je, i.length, f.isCircular);
          if (oe.error) return (X.push(oe.error), re());
          (oe.warning && X.push(oe.warning), (h = oe.op));
        }
        if (((y = Bt(i, d, p, h)), (!Me && i.length - y.length < 60) || (!Me && p < d && d - p < 60)))
          return (X.push("insufficient flanking sequence space to build primers"), re());
        if (Me) {
          let oe = ct.normalizeFlanks(d, p, i.length, !1);
          if (oe.error) return (X.push(oe.error), re());
          let ge = ct.flankError(oe.start, oe.end, i.length, !1, ct.MIN_FLANK_5, ct.MIN_FLANK_3);
          if (ge) return (X.push(ge), re());
        }
        return (tl(h, y, d, p, je, A, D, q, ue, _e, Me, null, L, I, X), re());
      } else if (L == "codon") {
        t(59, (_l = !1));
        let Le = jt.positionError("ORF start", Q, i.length);
        if (Le) return (X.push(Le), re());
        let je = Number(Q),
          oe = [],
          ge = R.toUpperCase()
            .trim()
            .split(/[\s,;]+/),
          xe = "",
          et = 0,
          Ct = 0,
          Tt = "",
          wt,
          mt = "",
          ft,
          Sl,
          St = "";
        if (k) {
          t(7, (A = !0));
          let Ye = [],
            Pe = [];
          for (let ze = 0; ze < ge.length; ++ze) {
            if (ge[ze] == "") continue;
            let en = jt.checkCodonOp(ge[ze], je, i.length, !Me);
            if (en) {
              X.push(en);
              continue;
            }
            if (ge[ze].toUpperCase().match(/[A-Z*]\d+[A-Z*]/)) {
              ((oe = ge[ze]
                .toUpperCase()
                .replace(/([A-Z*])(\d+)([A-Z*])/, "$1:$2:$3")
                .split(":")),
                oe.length == 4 && oe[3].match(/[ACGT]{3}/) && Te.translate(oe[3]) == oe[2]
                  ? (St = oe[3])
                  : ((St = ""), oe.length == 4 && oe[2] === "X" && X.push(ge[ze] + ": triplet ignored for X (NNK is used)")));
              let dl = je + (Number(oe[1]) - 1) * 3 - 1;
              if (f.isCircular) ((et = dl % i.length), (Ct = (et + 4) % i.length));
              else {
                let gt = ct.normalizeFlanks(dl, dl + 4, i.length, !1);
                if (gt.error) {
                  X.push("skipping op [" + ge[ze] + "] - " + gt.error);
                  continue;
                }
                ((et = gt.start), (Ct = gt.end));
              }
              if (
                ((Tt = "replace"),
                (wt = ge[ze]),
                (ft = ct.codonAt(i, dl, f.isCircular)),
                (Sl = Te.translate(ft)),
                St ? (xe = St) : x == "mindist" ? (xe = Xt(oe[2], ft)) : (xe = rt(oe[2])),
                Sl != oe[0])
              ) {
                if (ml()) {
                  X.push(jl(ge[ze], oe[0], Sl, oe[1]));
                  continue;
                }
                X.push(ge[ze] + ": " + Sl + " found at " + oe[1] + " instead of " + oe[0]);
              }
              (Ye.push({ start: dl, end: dl + 4, codon: xe, id: ge[ze], target: oe[2], triplet: St || null }), Pe.push(ge[ze]));
            } else {
              ge[ze] != "" && X.push("skipping op [" + ge[ze] + "] - unable to parse");
              continue;
            }
          }
          if (Ye.length == 0) return re();
          Ye.sort((ze, en) => ze.start - en.start);
          let $e = ct.mergeCodonOps(i, Ye, f.isCircular);
          if ($e.error) return (X.push($e.error), re());
          let yt = Ye.map(function (ze) {
              return { start: ze.start, end: ze.end, codon: ct.codonAt(i, ze.start, f.isCircular), id: ze.id };
            }),
            ss = ct.mergeCodonOps(i, yt, f.isCircular);
          if (!ss.error && $e.repseq.toLowerCase() === ss.repseq.toLowerCase())
            return (X.push("[" + Pe.join(" ") + "] no-op: merged replacement matches the existing sequence"), re());
          let fl = jt.replacementError($e.repseq, q === !0);
          if (fl) return (X.push("[" + Pe.join(" ") + "] " + fl), re());
          if (((et = $e.start), (Ct = $e.end), (xe = $e.repseq), (Tt = "replace"), (mt = Bt(i, et, Ct, Tt)), i.length - mt.length < 60))
            return (X.push("[" + Pe.join(" ") + "] insufficient flanking sequence space to build primers"), re());
          if (Me) {
            let ze = ct.flankError(et, Ct, i.length, !1, ct.MIN_FLANK_5, ct.MIN_FLANK_3);
            if (ze) return (X.push("[" + Pe.join(" ") + "] " + ze), re());
          }
          Se = tl(
            Tt,
            mt,
            et,
            Ct,
            xe,
            A,
            Pe.join("_"),
            q,
            ue,
            _e,
            Me,
            {
              kind: "merge",
              subs: Ye.map(function (ze) {
                return { id: ze.id, start: ze.start, end: ze.end, target: ze.target, triplet: ze.triplet };
              }),
            },
            void 0,
            I,
            X,
          );
        } else {
          let Ye = jt.duplicateOpsWarning(ge);
          for (let Pe = 0; Pe < ge.length; ++Pe) {
            if (ge[Pe] == "") continue;
            let $e = jt.checkCodonOp(ge[Pe], je, i.length, !Me);
            if ($e) {
              X.push($e);
              continue;
            }
            if (Ye[Pe]) {
              X.push("skipping op [" + ge[Pe] + "] - " + Ye[Pe]);
              continue;
            }
            if (ge[Pe].toUpperCase().match(/[A-Z*]\d+[A-Z*]|[A-Z*]\d+[A-Z*]/)) {
              ((oe = ge[Pe].toUpperCase()
                .replace(/([A-Z*])(\d+)([A-Z*])/, "$1:$2:$3")
                .split(":")),
                oe.length == 4 && oe[3].match(/[ACGT]{3}/) && Te.translate(oe[3]) == oe[2]
                  ? (St = oe[3])
                  : ((St = ""), oe.length == 4 && oe[2] === "X" && X.push(ge[Pe] + ": triplet ignored for X (NNK is used)")));
              let yt = je + (Number(oe[1]) - 1) * 3 - 1;
              if (f.isCircular) ((et = yt % i.length), (Ct = (et + 4) % i.length));
              else {
                let ss = ct.normalizeFlanks(yt, yt + 4, i.length, !1);
                if (ss.error) {
                  X.push("skipping op [" + ge[Pe] + "] - " + ss.error);
                  continue;
                }
                ((et = ss.start), (Ct = ss.end));
              }
              if (
                ((Tt = "replace"),
                (wt = ge[Pe]),
                (ft = ct.codonAt(i, yt, f.isCircular)),
                (Sl = Te.translate(ft)),
                St ? (xe = St) : x == "mindist" ? (xe = Xt(oe[2], ft)) : (xe = rt(oe[2])),
                xe.toLowerCase() === ft.toLowerCase())
              ) {
                X.push("skipping op [" + ge[Pe] + "] - no-op: codon " + oe[1] + " is already " + ft.toUpperCase());
                continue;
              }
              if (Sl != oe[0]) {
                if (ml()) {
                  X.push(jl(ge[Pe], oe[0], Sl, oe[1]));
                  continue;
                }
                X.push(ge[Pe] + ": " + Sl + " found at " + oe[1] + " instead of " + oe[0]);
              }
              if (((mt = Bt(i, et, Ct, Tt)), i.length - mt.length < 60)) {
                X.push("[" + ge[Pe] + "] insufficient flanking sequence space to build primers");
                continue;
              }
              if (Me) {
                let ss = ct.flankError(et, Ct, i.length, !1, ct.MIN_FLANK_5, ct.MIN_FLANK_3);
                if (ss) {
                  X.push("[" + ge[Pe] + "] " + ss);
                  continue;
                }
              }
              Se = tl(
                Tt,
                mt,
                et,
                Ct,
                xe,
                A,
                (D ? D + "_" : "") + wt,
                q,
                ue,
                _e,
                Me,
                { kind: "codon", target: oe[2], triplet: St || null },
                void 0,
                I,
                X,
              );
            } else continue;
          }
        }
        return re();
      } else if (L == "batch") {
        t(59, (_l = !0));
        let Le = 1,
          je = [],
          oe = Xe.split(/\r?\n/),
          ge,
          xe = "",
          et = 0,
          Ct = 0,
          Tt = "",
          wt,
          mt = "",
          ft,
          Sl,
          St = "",
          Ye = null,
          Pe = new Set();
        for (let $e = 0; $e < oe.length; ++$e) {
          if (((ge = oe[$e].trim()), ge == "")) continue;
          if (((Ye = null), ge.toUpperCase().startsWith("ORF"))) {
            se = ge.split(/[:\s]+/);
            let fl = jt.positionError("ORF start", se[1], i.length);
            if (fl) {
              (X.push("skipping line [" + ge + "] - " + fl), (Le = NaN));
              continue;
            }
            Le = Number(se[1]);
            continue;
          }
          se = ge.toUpperCase().split(/[,\s;]+/);
          let yt = jt.batchLineError(se);
          if (yt) {
            X.push("skipping line [" + ge + "] - " + yt);
            continue;
          }
          let ss = se.length < 3 ? Le + "|" + jt.normalizeOpText(se[0]) : jt.normalizeOpText(ge);
          if (Pe.has(ss)) {
            X.push("skipping line [" + ge + "] - duplicate of an earlier op");
            continue;
          }
          if ((Pe.add(ss), se.length < 3)) {
            if (se.length == 1) {
              let fl = jt.checkCodonOp(se[0], Le, i.length, !Me);
              if (fl) {
                X.push(fl);
                continue;
              }
            }
            if (se.length == 1 && se[0].toUpperCase().match(/[A-Z*]\d+[A-Z*]|[A-Z*]\d+[A-Z*]/)) {
              ((je = se[0]
                .toUpperCase()
                .replace(/([A-Z*])(\d+)([A-Z*])/, "$1:$2:$3")
                .split(":")),
                je.length == 4 && je[3].match(/[ACGT]{3}/) && Te.translate(je[3]) == je[2]
                  ? (St = je[3])
                  : ((St = ""), je.length == 4 && je[2] === "X" && X.push(oe[$e] + ": triplet ignored for X (NNK is used)")));
              let fl = Le + (Number(je[1]) - 1) * 3 - 1;
              if (f.isCircular) ((et = fl % i.length), (Ct = (et + 4) % i.length));
              else {
                let ze = ct.normalizeFlanks(fl, fl + 4, i.length, !1);
                if (ze.error) {
                  X.push("skipping op [" + se[0] + "] - " + ze.error);
                  continue;
                }
                ((et = ze.start), (Ct = ze.end));
              }
              if (
                ((Tt = "replace"),
                (wt = se[0]),
                (ft = ct.codonAt(i, fl, f.isCircular)),
                (Sl = Te.translate(ft)),
                St ? (xe = St) : x == "mindist" ? (xe = Xt(je[2], ft)) : (xe = rt(je[2])),
                xe.toLowerCase() === ft.toLowerCase())
              ) {
                X.push("skipping line [" + ge + "] - no-op: codon " + je[1] + " is already " + ft.toUpperCase());
                continue;
              }
              (Sl != je[0] && X.push(oe[$e] + ": " + Sl + " found at " + je[1] + " instead of " + je[0]),
                (Ye = { kind: "codon", target: je[2], triplet: St || null }));
            } else {
              X.push("skipping line [" + ge + "] - unable to parse");
              continue;
            }
          } else {
            let fl =
              jt.positionError("start", se[1], i.length) ||
              jt.positionError("end", se[2], i.length) ||
              jt.replacementError(se[3], q === !0);
            if (fl) {
              X.push("skipping line [" + ge + "] - " + fl);
              continue;
            }
            if (((wt = se[0]), f.isCircular)) ((et = Number(se[1]) % i.length), (Ct = Number(se[2]) % i.length));
            else {
              let en = ct.normalizeFlanks(Number(se[1]), Number(se[2]), i.length, !1);
              if (en.error) {
                X.push("skipping line [" + ge + "] - " + en.error);
                continue;
              }
              ((et = en.start), (Ct = en.end));
            }
            xe = jt.stripReplacement(se[3] || "");
            let ze = ct.classifyOp(et, Ct, xe, i.length, f.isCircular);
            if (ze.error) {
              X.push("skipping line [" + ge + "] - " + ze.error);
              continue;
            }
            (ze.warning && X.push(ge + ": " + ze.warning), (Tt = ze.op));
          }
          if (((mt = Bt(i, et, Ct, Tt)), i.length - mt.length < 60)) {
            X.push(ge + ": insufficient flanking sequence space to build primers");
            continue;
          }
          if (Me) {
            let fl = ct.flankError(et, Ct, i.length, !1, ct.MIN_FLANK_5, ct.MIN_FLANK_3);
            if (fl) {
              X.push(ge + ": " + fl);
              continue;
            }
          }
          try {
            Se = tl(Tt, mt, et, Ct, xe, A, (D ? D + "_" : "") + wt, q, ue, _e, Me, Ye, void 0, I, X);
          } catch {
            X.push("skipping line [" + ge + "] - unable to build primers");
          }
        }
        return re();
      }
    },
    zl = function (I, X) {
      return (
        "Unexpected error: unable to " +
        I +
        " from the current input (" +
        (X && X.message) +
        "). Check the sequence and mutation entries for invalid characters."
      );
    },
    rl = function () {
      try {
        var I = Nt() || { primers: [], warnings: [] },
          X = (I.warnings || []).slice(),
          re = Wl(I.primers, X);
        return (
          re.forEach(function (se, Se) {
            se.pairIndex = Se + 1;
          }),
          { primers: re, warnings: X }
        );
      } catch (se) {
        return { primers: [], warnings: [zl("build primers", se)] };
      }
    },
    rs = null,
    el = function (I, X, re, se, Se, ...Me) {
      try {
        var Le = { tick: I, inputmode: X, codonops: re, rawbatchinput: se, seq: Se },
          je = ct.buildAction(rs, Le);
        return (
          (rs = Le),
          t(26, (M = 0)),
          t(32, (z = [])),
          je == "clear"
            ? (t(33, (pe = Se.length < 60)), t(34, (Y = !!jt.sequenceLengthError(Se.length))), { primers: [], warnings: [] })
            : rl()
        );
      } catch (oe) {
        return { primers: [], warnings: [zl("build primers", oe)] };
      }
    },
    os = function (I, X, re) {
      try {
        return Bl(I, X, re);
      } catch (se) {
        return { primers: [], warnings: [zl("build NEBuilder primers", se)] };
      }
    },
    tl = function (I, X, re, se, Se, Me, Le, je, oe, ge, xe, et, Ct, Tt, wt) {
      ((Tt = Tt || []), (wt = wt || []), (X = X || ""));
      var mt = re - 1,
        ft = se - 1,
        Sl,
        St = 10,
        Ye,
        Pe = function (dl, gt) {
          var hn = Math.min(dl, gt) + 1;
          return (hn > 72 && (hn = 72), hn);
        },
        $e = {};
      (xe || ((mt = mt % i.length), (ft = ft % i.length)),
        (oe = oe < 15 ? 15 : oe),
        (ge = ge < 55 ? 55 : ge),
        (ge = ge > 72 ? 72 : ge),
        ($e.primerMinLen = oe),
        ($e.primerMinTm = ge),
        ($e.primerMaxTm = 72));
      var yt,
        ss = [Le, mt, ft, Se].join(",");
      if ((Le || (Le = "primer"), !(u && re >= 0 && se >= 0) || I == "not_ready" || X == null)) return Tt;
      if (xe) {
        let dl = ct.flankError(re, se, i.length, !1, ct.MIN_FLANK_5, ct.MIN_FLANK_3);
        if (dl) return (wt.push(dl), Tt);
      }
      if (i.length - X.length < 60) return (wt.push("insufficient flanking sequence space to build primers"), Tt);
      ((_ = je ? [Se] : Te.expandDNASeq(Se)), _.length == 0 && (_ = [""]));
      for (let dl = 0; dl < _.length; ++dl) {
        try {
          ((Ye = _[dl]),
            (yt = Jt.describeOp({ seqop: I, start: re, end: se, newseq: Ye, seq: i, circular: !xe, seqName: u.id }) + " (" + Le + ")"));
        } catch {
          wt.push("Unexpected error preparing for operation.");
        }
        if (I == "delete")
          try {
            if (u && re >= 0 && se >= 0) {
              X === null && (X = "");
              var fl = ft > mt ? ft - mt : ft - mt + i.length,
                ze = fl - 1,
                en = i.length - ze;
              if (en < 62) {
                let gt = "deleting " + ze + " bp leaves only " + en + " bp, too little to design primers";
                return (wt.push(gt), Tt);
              }
              if (f.isCircular) u.subseq = [ft, mt];
              else {
                let gt = f.seq.slice(ft) + f.seq.slice(0, mt + 1);
                (u.updateSrcseq(gt), (u.subseq = [0, gt.length - 1]));
              }
              if (((u.srctype = "pcr"), u.autogenPrimers($e), !f.isCircular)) {
                let gt = i.length - ft,
                  hn = mt + 1;
                if (u.fwdPrimer.anneal.length > gt || u.revPrimer.anneal.length > hn)
                  throw new Error("cannot reach min Tm " + ge + " without crossing the end of the partial sequence");
              }
              Tt.push({
                selstart: re,
                selend: se,
                locf: ft,
                locr: mt,
                seqf: u.fwdPrimer.seq,
                seqr: u.revPrimer.seq,
                lenf: u.fwdPrimer.seq.length,
                lenr: u.revPrimer.seq.length,
                tmf: u.fwdPrimer.tm,
                tmr: u.revPrimer.tm,
                ta: Pe(u.fwdPrimer.tm, u.revPrimer.tm),
                repseq: _[dl],
                seqfpflap: "",
                seqrpflap: "",
                seqfanneal: u.fwdPrimer.anneal.toUpperCase(),
                seqranneal: u.revPrimer.anneal.toUpperCase(),
                seqrnbanneal: "",
                description: yt,
                seqop: I,
                idprefix: Le,
                op: et || null,
                pad: 0,
                batch_op: ss,
              });
            }
          } catch (gt) {
            gt.message && gt.message.indexOf("cannot reach") !== -1
              ? wt.push(Le + " (" + re + "-" + se + "): " + gt.message)
              : wt.push("Unexpected error in delete operation.");
          }
        else if (I == "insert" || (I == "replace" && (Me || Ye.length >= 6 || X.length >= 6 || !/^[acgt]*$/i.test(Ye))))
          try {
            if (u && re >= 0 && se >= 0 && Ye) {
              if (f.isCircular) u.subseq = [ft, mt];
              else {
                let gt = f.seq.slice(ft) + f.seq.slice(0, mt + 1);
                (u.updateSrcseq(gt), (u.subseq = [0, gt.length - 1]));
              }
              if (((u.srctype = "pcr"), u.autogenPrimers($e), (Sl = Se.length / 2), u.autogenPrimers($e), !f.isCircular)) {
                let gt = i.length - ft,
                  hn = mt + 1;
                if (u.fwdPrimer.anneal.length > gt || u.revPrimer.anneal.length > hn)
                  throw new Error("cannot reach min Tm " + ge + " without crossing the end of the partial sequence");
              }
              Tt.push({
                selstart: re,
                selend: se,
                locf: ft,
                locr: mt,
                seqf: Ye.length <= 6 ? Ye + u.fwdPrimer.seq : Ye.slice(Sl) + u.fwdPrimer.seq,
                seqr: Ye.length <= 6 ? u.revPrimer.seq : Te.revcomp(Ye.slice(0, Sl)) + u.revPrimer.seq,
                lenf: u.fwdPrimer.seq.length,
                lenr: u.revPrimer.seq.length,
                tmf: u.fwdPrimer.tm,
                tmr: u.revPrimer.tm,
                ta: Pe(u.fwdPrimer.tm, u.revPrimer.tm),
                repseq: _[dl],
                seqfpflap: Ye.length <= 6 ? Ye.toLowerCase() : Ye.slice(Sl).toLowerCase(),
                seqrpflap: Ye.length <= 6 ? "" : Te.revcomp(Ye.slice(0, Sl)).toLowerCase(),
                seqfanneal: u.fwdPrimer.anneal.toUpperCase(),
                seqranneal: u.revPrimer.anneal.toUpperCase(),
                seqrnbanneal: Ne.revcomp(u.fwdPrimer.anneal.toUpperCase()),
                description: yt,
                seqop: I,
                idprefix: Le,
                op: et || null,
                pad: 0,
                batch_op: ss,
              });
            }
          } catch (gt) {
            gt.message && gt.message.indexOf("cannot reach") !== -1
              ? wt.push(Le + " (" + re + "-" + se + "): " + gt.message)
              : wt.push("Unexpected error in replace/insert operation.");
          }
        else if (I == "replace")
          try {
            if (u && re >= 0 && se >= 0 && Ye && X) {
              let gt = 1 / 0,
                hn = 1 / 0;
              if (f.isCircular) u.subseq = [mt - St + 1, mt - St];
              else {
                let Mi = f.seq.slice(mt - St + 1) + f.seq.slice(0, mt - St + 1);
                (u.updateSrcseq(Mi), (u.subseq = [0, Mi.length - 1]), (gt = i.length - ft), (hn = mt - St + 1));
              }
              u.srctype = "pcr";
              let ns = Ne.calcSDMPrimers(u.template, oe, ge, St, X, Ye, gt, hn);
              ((ns.ta = Math.min(ns.tmf, ns.tmr) + 1),
                ns.ta > 72 && (ns.ta = 72),
                Tt.push({
                  selstart: re,
                  selend: se,
                  locf: (((mt - St + 1) % i.length) + i.length) % i.length,
                  locr: (((mt - St) % i.length) + i.length) % i.length,
                  seqf: ns.pfwd,
                  seqr: ns.prev,
                  lenf: ns.pfwd.length,
                  lenr: ns.prev.length,
                  tmf: ns.tmf,
                  tmr: ns.tmr,
                  ta: ns.ta,
                  repseq: _[dl],
                  seqfpflap: "",
                  seqrpflap: "",
                  seqfanneal: ns.pfwd.slice(0, St).toUpperCase() + _[dl].toLowerCase() + ns.pfwd.slice(St + _[dl].length).toUpperCase(),
                  seqranneal: ns.prev.toUpperCase(),
                  seqrnbanneal: Ne.revcomp(
                    ns.pfwd.slice(0, St).toUpperCase() + _[dl].toLowerCase() + ns.pfwd.slice(St + _[dl].length).toUpperCase(),
                  ),
                  description: yt,
                  seqop: I,
                  idprefix: Le,
                  op: et || null,
                  pad: St,
                  batch_op: ss,
                }));
            }
          } catch (gt) {
            gt.message && gt.message.indexOf("cannot reach") !== -1
              ? wt.push(Le + " (" + re + "-" + se + "): " + gt.message)
              : wt.push("Unexpected error in replace operation.");
          }
      }
      return Tt;
    };
  ((Ne.revcomp = function (I) {
    var X = "ACGTUKMSWRYBDHVNacgtukmswrybdhvn-",
      re = "TGCAAMKSWYRVHDBNtgcaamkswyrvhdbn-",
      se = I.split("").reverse(),
      Se,
      Me;
    for (Se = 0; Se < se.length; ++Se) {
      if (((Me = X.indexOf(se[Se])), Me === -1)) throw { name: "InvalidCharacter", message: "A non IUPAC base was encountered" };
      se[Se] = re.charAt(Me);
    }
    return se.join("");
  }),
    (Ne.rev = function (I) {
      return I.split("").reverse().join("");
    }),
    (Ne.comp = function (I) {
      var X = "ACGTUKMSWRYBDHVNacgtukmswrybdhvn-",
        re = "TGCAAMKSWYRVHDBNtgcaamkswyrvhdbn-",
        se = I.split(""),
        Se,
        Me;
      for (Se = 0; Se < se.length; ++Se) {
        if (((Me = X.indexOf(se[Se])), Me === -1)) throw { name: "InvalidCharacter", message: "A non IUPAC base was encountered" };
        se[Se] = re.charAt(Me);
      }
      return se.join("");
    }),
    (Ne.tm = function (I, X) {
      var re = 0.5,
        se = "Q5",
        Se = 150,
        Me = 4;
      X = X || Ne.revcomp(I);
      var Le = Ne.createTmCalc({ seq: I, ct: re, salt: Se, compseq: X });
      switch (se) {
        case "Phusion":
        case "Phusion Hot Start Flex":
          Me = 1;
          break;
        default:
          Me = 4;
      }
      Le.setMethod(Me);
      var je = Le.Tm(),
        oe = je.tm;
      return oe;
    }),
    (Ne.calcSDMPrimers = function (I, X, re, se, Se, Me, Le, je) {
      ((X = X < 15 ? 15 : X),
        (re = re < 55 ? 55 : re),
        (re = re > 72 ? 72 : re),
        (Le = Le === void 0 ? 1 / 0 : Le),
        (je = je === void 0 ? 1 / 0 : je));
      var oe = {},
        ge = se,
        xe = X,
        et = ge,
        Ct = xe,
        Tt = 63,
        wt,
        mt = Li.alignDNAglobal,
        ft = {};
      ((ft.mat = {
        a: { a: 2, c: -1, g: 1, t: -1 },
        c: { a: -1, c: 2, g: -1, t: 1 },
        g: { a: 1, c: -1, g: 2, t: -1 },
        t: { a: -1, c: 1, g: -1, t: 2 },
      }),
        (ft.gap = -2.5),
        (ft.term = -2.5),
        (ft.extend = -1));
      var Sl = /^[acgt]+$/i,
        St = 10,
        Ye =
          Me.length === Se.length && Me.length < St && Sl.test(Me) && Sl.test(Se)
            ? { alnSeq1: Me.toLowerCase(), alnSeq2: Se.toLowerCase() }
            : mt(new Li.Alignment("global", Me, Se, ft));
      wt = Ne.revcomp(I);
      var Pe = se + Se.length;
      if (((Le = Math.min(Le, I.length - Pe)), (je = Math.min(je, I.length)), ge > Le))
        throw new Error("cannot reach min primer length " + ge + ": only " + Le + " bases available 3' of the mutation");
      if (xe > je) throw new Error("cannot reach min primer length " + xe + ": only " + je + " bases available 5' of the mutation");
      ((oe.pfwd = I.slice(0, se) + Ye.alnSeq1 + I.slice(Pe, Pe + ge)), (oe.prev = wt.slice(0, xe)));
      for (var $e = Ne.tm(oe.pfwd, Ne.revcomp(I.slice(0, se) + Ye.alnSeq2 + I.slice(Pe, Pe + ge))); $e < re; ) {
        if ((Ne.dmsg("tmf " + $e + " < " + re), ge >= Le))
          throw new Error("cannot reach min Tm: only " + Le + " bases available 3' of the mutation");
        (++ge,
          (oe.pfwd = I.slice(0, se) + Ye.alnSeq1 + I.slice(Pe, Pe + ge)),
          ($e = Ne.tm(oe.pfwd, Ne.revcomp(I.slice(0, se) + Ye.alnSeq2 + I.slice(Pe, Pe + ge)))),
          (et = ge));
      }
      for (; $e < Tt && ge < Le && oe.pfwd.search("[GgCc]$") === -1; )
        (Ne.dmsg("extending fwd to terminal g/c"),
          ++ge,
          (oe.pfwd = I.slice(0, se) + Ye.alnSeq1 + I.slice(Pe, Pe + ge)),
          ($e = Ne.tm(oe.pfwd, Ne.revcomp(I.slice(0, se) + Ye.alnSeq2 + I.slice(Pe, Pe + ge)))));
      oe.pfwd.search("[GCgc]$") === -1 &&
        (Ne.dmsg("reverting to minimal good fwd primer"),
        (oe.pfwd = I.slice(0, se) + Ye.alnSeq1 + I.slice(Pe, Pe + et)),
        ($e = Ne.tm(oe.pfwd, Ne.revcomp(I.slice(0, se) + Ye.alnSeq2 + I.slice(Pe, Pe + et)))),
        (ge = et));
      for (var yt = Ne.tm(oe.prev); yt < re; ) {
        if ((Ne.dmsg("tmr " + yt + " < " + re), xe >= je))
          throw new Error("cannot reach min Tm: only " + je + " bases available 5' of the mutation");
        ((oe.prev = wt.slice(0, ++xe)), (yt = Ne.tm(oe.prev)), (Ct = xe));
      }
      for (; yt < Tt && xe < je && oe.prev.search("[GgCc]$") === -1; )
        (Ne.dmsg("extending rev to terminal g/c"), (oe.prev = wt.slice(0, ++xe)), (yt = Ne.tm(oe.prev)));
      if (
        (oe.prev.search("[GgCc]$") === -1 &&
          (Ne.dmsg("reverting to minimal good rev primer"), (oe.prev = wt.slice(0, Ct)), (yt = Ne.tm(oe.prev)), (xe = Ct)),
        $e > yt)
      ) {
        for (Ne.dmsg("tmf > tmr"); $e - yt > 5 && xe < je; )
          (Ne.dmsg("tmf - tmr > 5"), (oe.prev = wt.slice(0, ++xe)), (yt = Ne.tm(oe.prev)));
        for (Ct = xe; yt < $e && xe < je && oe.prev.search("[GgCc]$") === -1; )
          if ((Ne.dmsg("tmr < tmf but does not end in g/c"), (oe.prev = wt.slice(0, ++xe)), (yt = Ne.tm(oe.prev)), yt < $e)) Ct = xe;
          else {
            ((oe.prev = wt.slice(0, Ct)), (yt = Ne.tm(oe.prev)));
            break;
          }
      }
      if (yt > $e) {
        for (Ne.dmsg("tmr > tmf"); yt - $e > 5 && ge < Le; )
          (Ne.dmsg("tmf - tmr > 5"),
            ++ge,
            (oe.pfwd = I.slice(0, se) + Ye.alnSeq1 + I.slice(Pe, Pe + ge)),
            ($e = Ne.tm(oe.pfwd, Ne.revcomp(I.slice(0, se) + Ye.alnSeq2 + I.slice(Pe, Pe + ge)))));
        for (et = ge; $e < yt && ge < Le && oe.pfwd.search("[GgCc]$") === -1; )
          if (
            (Ne.dmsg("tmf < tmr but does not end in g/c"),
            ++ge,
            (oe.pfwd = I.slice(0, se) + Ye.alnSeq1 + I.slice(Pe, Pe + ge)),
            ($e = Ne.tm(oe.pfwd, Ne.revcomp(I.slice(0, se) + Ye.alnSeq2 + I.slice(Pe, Pe + ge)))),
            $e < yt)
          )
            et = ge;
          else {
            ((oe.pfwd = I.slice(0, se) + Ye.alnSeq1 + I.slice(Pe, Pe + et)),
              ($e = Ne.tm(oe.pfwd, Ne.revcomp(I.slice(0, se) + Ye.alnSeq2 + I.slice(Pe, Pe + et)))));
            break;
          }
      }
      return ((oe.tmf = $e), (oe.tmr = yt), (oe.pfwd = oe.pfwd.replace(/-/g, "")), (oe.prev = oe.prev.replace(/-/g, "")), oe);
    }));
  var qt = function (I) {
      var X = v || "";
      return !I || I == "" ? X : (Xl(), String(I).replace(/\s/g, ""));
    },
    gl = function (I) {
      var X = v || "";
      return !I || I == "" ? X : (Fl(), String(I).replace(/\s/g, ""));
    },
    Fl = function () {
      t(5, (b = ""));
    },
    Xl = function () {
      t(6, (w = ""));
    },
    Ze = "",
    vl = null,
    Ql = !1,
    xl = function (I) {
      if (vl !== null) {
        if (I === vl) {
          ((vl = null), Ql && t(19, (te += 1)));
          return;
        }
        if (!I || !I.trim()) return;
        vl = null;
      }
      (t(23, (Ze = "")), t(2, (d = void 0)), t(3, (p = void 0)), t(4, (v = "")), t(13, (R = "")), t(14, (Q = 1)));
    },
    lt = [],
    st = 0,
    Ml = function (I) {
      const X = i.length;
      let re = I,
        se = I + Ze.length + 1;
      return (f.isCircular && (re == 0 && (re = X), (se = se % X), se == 0 && (se = X)), [re, se]);
    },
    Kt = function () {
      if (Ze != "") {
        var I = new RegExp(jt.escapeRegExp(Ze), "ig"),
          X;
        for (t(58, (lt = [])); (X = I.exec(i)) !== null; ) lt.push(Number(X.index));
        lt.length > 0
          ? ((st = 0), t(2, ([d, p] = Ml(lt[st])), d, t(3, p)), Rt.setScrollPos(d))
          : alert("The target sequence was not found.");
      }
    },
    ys = function () {
      lt.length > 0 && ((st = (st + 1) % lt.length), t(2, ([d, p] = Ml(lt[st])), d, t(3, p)), Rt.setScrollPos(d));
    },
    Jl = function () {
      lt.length > 0 && ((st = st > 0 ? st - 1 : lt.length - 1), t(2, ([d, p] = Ml(lt[st])), d, t(3, p)), Rt.setScrollPos(d));
    },
    as = function () {
      if (Ze != "") {
        var I = new RegExp(jt.escapeRegExp(Ze), "ig"),
          X;
        for (t(58, (lt = [])); (X = I.exec(i)) !== null; ) lt.push(Number(X.index));
        lt.length > 0
          ? ((st = 0), t(2, ([d, p] = Ml(lt[st])), d, t(3, p)), Lt.setScrollPos(d))
          : alert("The target sequence was not found.");
      }
    },
    Pt = function () {
      lt.length > 0 && ((st = (st + 1) % lt.length), t(2, ([d, p] = Ml(lt[st])), d, t(3, p)), Lt.setScrollPos(d));
    },
    Ht = function () {
      lt.length > 0 && ((st = st > 0 ? st - 1 : lt.length - 1), t(2, ([d, p] = Ml(lt[st])), d, t(3, p)), Lt.setScrollPos(d));
    },
    Zl = function () {
      let I = Q - 1,
        X = Q;
      for (const re of ct.orfStarts(i))
        if (re < I) X = re + 1;
        else break;
      Q != X && t(14, (Q = X));
    },
    cs = function () {
      let I = Q - 1,
        X = Q;
      for (const re of ct.orfStarts(i))
        if (re > I) {
          X = re + 1;
          break;
        }
      Q != X && t(14, (Q = X));
    },
    Qt = function () {
      (t(23, (Ze = "")), (st = 0), t(58, (lt = [])));
    },
    As = function () {
      (t(4, (v = "")), t(5, (b = "")), t(6, (w = "")));
    },
    _l = !0;
  let Rt, Lt;
  var Bl = function (I, X, re) {
    return !X || !I || I.length == 0 ? { primers: [], warnings: [] } : ct.nebuilderFragments(I, i.length, re);
  };
  let bl = "Substitution";
  function Ut(I) {
    (t(24, (bl = I)),
      bl == "Substitution"
        ? t(7, (A = !1))
        : bl == "Deletion"
          ? (t(7, (A = !0)), t(4, (v = "")))
          : bl == "Insertion" && ((y = ""), t(7, (A = !0)), t(3, (p = f.isCircular ? (d % i.length) + 1 : d + 1))));
  }
  function il(I) {
    const X = document.getElementById(I);
    if (!X) return null;
    const re = window.getSelection();
    let se, Se, Me;
    if (re.rangeCount > 0) {
      re.focusNode;
      const Le = re.getRangeAt(0);
      return X.contains(Le.commonAncestorContainer)
        ? ((se = Le.startOffset), (Se = re.toString()), (Me = i.slice(se, se + Se.length)), re.removeAllRanges(), [se, Me])
        : null;
    }
    return (re.removeAllRanges(), null);
  }
  let ll = 0,
    ql = null;
  function Hl(I, X) {
    let re = !f.isCircular,
      se = I,
      Se = I + X + 1;
    (re || (se == 0 && (se = i.length), (Se = Se % i.length), Se == 0 && (Se = i.length)), t(2, (d = se)), t(3, (p = Se)));
  }
  function Ul() {
    if (t(97, (a = il("legacy_fwd_seq")))) (a != "" && (Qt(), t(23, (Ze = a[1]))), Hl(a[0], a[1].length));
    else if (t(97, (a = il("legacy_rev_seq")))) (a != "" && (Qt(), t(23, (Ze = a[1]))), Hl(a[0], a[1].length));
    else if (t(97, (a = il("indel_fwd_seq")))) (a != "" && (Qt(), t(23, (Ze = a[1]))), Hl(a[0], a[1].length));
    else if (t(97, (a = il("indel_rev_seq")))) (a != "" && (Qt(), t(23, (Ze = a[1]))), Hl(a[0], a[1].length));
    else if (t(97, (a = il("codon_fwd_seq")))) {
      if (a[1].length <= 3) {
        let I = (Q + a[0]) % i.length;
        ((I = I == 0 ? i.length : I), I != Q && t(14, (Q = I)));
      }
    } else
      t(97, (a = il("codon_orf_fwd")))
        ? a[1].length <= 3 && Gs(a[0], Q)
        : t(97, (a = il("codon_aa_fwd"))) && a[1].length <= 3 && Gs(a[0], Q);
  }
  const us = (I) => {
    (ll++,
      ll === 1
        ? (ql = setTimeout(() => {
            ((ll = 0), Ul());
          }, Mc))
        : ll === 2 && (clearTimeout(ql), (ll = 0)));
  };
  (document.addEventListener("click", us),
    Dn(() => {
      (clearTimeout(Vt), document.removeEventListener("click", us), clearTimeout(ql));
    }));
  function It() {
    (t(13, (R = "")), t(19, (te += 1)));
  }
  function Ks() {
    t(14, (Q = 1));
  }
  function zs() {
    (t(49, (Be.returnValue = ""), Be), t(6, (w = "")), Be.showModal());
  }
  function ol(I) {
    let X = [I.srcaa, I.cdn.toString(), I.rep].join("");
    (I.target && (X += ":" + I.target.toLowerCase()), t(13, (R += (R.length > 0 ? " " : "") + X)));
  }
  function Gs(I, X) {
    let re = Math.floor(I / 3) + 1,
      se = (X + (re - 1) * 3 - 1) % i.length,
      Se = (se + 3) % i.length,
      Me = se < Se ? i.slice(se, Se) : i.slice(se) + i.slice(0, Se),
      Le = Te.translate(Me);
    (t(55, (Ve.srcaa = Le), Ve),
      t(55, (Ve.srcaa3 = Te.AMINO_ACID_OLC[Le]), Ve),
      t(55, (Ve.srccdn = Me.toUpperCase()), Ve),
      t(55, (Ve.cdn = re), Ve),
      t(55, (Ve.orfstart = X), Ve),
      x == "mindist" ? t(55, (Ve.order = "cdn"), Ve) : t(55, (Ve.order = "cut"), Ve),
      zs());
  }
  const xt = go("sdm");
  (xt &&
    (({
      inputmode: L,
      codonops: R,
      orfoffset: Q,
      selstart: d,
      selend: p,
      repseq: v,
      rawbatchinput: Xe,
      tailsonly: A,
      keepDegen: q,
      mergeMutations: k,
      nebuilder: H,
      cdnsort: x,
      usermintm: _e,
      userminlen: ue,
      opid: D,
      filename: E,
      pfilename: P,
      sfilename: K,
      filename2: G,
      filename3: O,
    } = xt),
    (vl = xt.seq),
    (Ql = xt.hadPrimers),
    (ae = xt.opsFrame),
    (W = !0),
    (he = xt.userminlenSet)),
    Dn(() => {
      mo("sdm", {
        ...{
          inputmode: L,
          codonops: R,
          orfoffset: Q,
          selstart: d,
          selend: p,
          repseq: v,
          rawbatchinput: Xe,
          tailsonly: A,
          keepDegen: q,
          mergeMutations: k,
          nebuilder: H,
          cdnsort: x,
          usermintm: _e,
          userminlen: ue,
          opid: D,
          filename: E,
          pfilename: P,
          sfilename: K,
          filename2: G,
          filename3: O,
        },
        seq: i,
        hadPrimers: T.length > 0,
        opsFrame: ae,
        userminlenSet: he,
      });
    }));
  const tn = [[], []],
    Fs = () => Ie.showModal();
  function wl(I) {
    ((f = I), t(1, f));
  }
  const ln = () => Ce.showModal();
  function $l(I) {
    ((Ie = I), t(51, Ie));
  }
  function fs(I) {
    ((Ce = I), t(52, Ce));
  }
  function Rs(I) {
    ((it = I), t(48, it));
  }
  function es(I) {
    ((we = I), t(38, we));
  }
  function Cs(I) {
    ((fe = I), t(37, fe));
  }
  function ds(I) {
    (($ = I), t(36, $));
  }
  function yl(I) {
    ((We = I), t(50, We));
  }
  function un(I) {
    ((j = I), t(39, j));
  }
  function Vl(I) {
    ((F = I), t(40, F));
  }
  function Ys(I) {
    ((ut = I), t(47, ut));
  }
  function ts(I) {
    ((ee = I), t(41, ee));
  }
  function Ts(I) {
    ((V = I), t(42, V));
  }
  function Is(I) {
    ((me = I), t(43, me));
  }
  function ps(I) {
    ((_t = I), t(44, _t));
  }
  function Al(I) {
    ((bt = I), t(46, bt));
  }
  function Xs(I) {
    ((Je = I), t(45, Je));
  }
  function sl(I) {
    ((De = I), t(53, De));
  }
  function al() {
    ((w = Gn(this)), t(6, w));
  }
  const Ss = () => {
      (t(49, (Be.returnValue = w), Be), Be.close());
    },
    Cl = () => {
      (t(6, (w = "")), Be.close());
    };
  function ks(I) {
    ((Be = I), t(49, Be));
  }
  function Ds(I) {
    ((Ve = I), t(55, Ve));
  }
  const hs = () => {
    if (!(!w || !Be.returnValue)) {
      let I = $t(w);
      (t(55, (Ve.rep = Te.AMINO_ACID_TLC[I.toUpperCase()]), Ve), t(55, (Ve.target = w), Ve), ol(Ve));
    }
    t(6, (w = ""));
  };
  function Qs(I) {
    ((at = I), t(54, at));
  }
  function ms() {
    ((L = this.__value), t(12, L));
  }
  const xs = () => de(15),
    Ms = () => fe.showModal();
  function sn() {
    ((L = this.__value), t(12, L));
  }
  const Os = () => de(15),
    js = () => we.showModal();
  function Yt() {
    ((L = this.__value), t(12, L));
  }
  const cl = () => de(15),
    ls = () => $.showModal();
  function nn() {
    ((L = this.__value), t(12, L));
  }
  const gs = () => de(18),
    rn = () => it.showModal();
  function Bs() {
    ((D = this.value), t(11, D));
  }
  function Hs(I) {
    dt[I ? "unshift" : "push"](() => {
      ((Lt = I), t(61, Lt));
    });
  }
  function Js(I) {
    ((d = I), t(2, d));
  }
  function fn(I) {
    ((p = I), t(3, p));
  }
  const Zs = () => Ut("Substitution"),
    $s = () => Ut("Insertion"),
    ul = () => Ut("Deletion");
  function Tl() {
    ((Ze = this.value), t(23, Ze));
  }
  const El = () => {
    const I = /[^a-zA-Z]/g;
    (t(23, (Ze = Ze.replace(I, ""))), Ze == "" ? Qt() : as());
  };
  function Ae() {
    ((d = Us(this.value)), t(2, d));
  }
  function ot() {
    ((p = Us(this.value)), t(3, p));
  }
  function on() {
    ((v = this.value), t(4, v), t(239, qt), t(5, b), t(240, gl), t(6, w));
  }
  function ii() {
    ((b = Gn(this)), t(5, b));
  }
  function On() {
    ((Ze = this.value), t(23, Ze));
  }
  const ri = () => {
    const I = /[^a-zA-Z]/g;
    (t(23, (Ze = Ze.replace(I, ""))), Ze == "" ? Qt() : Kt());
  };
  function jn() {
    ((d = Us(this.value)), t(2, d));
  }
  function oi() {
    ((p = Us(this.value)), t(3, p));
  }
  function Bn(I) {
    dt[I ? "unshift" : "push"](() => {
      ((Rt = I), t(60, Rt));
    });
  }
  function ai() {
    ((v = this.value), t(4, v), t(239, qt), t(5, b), t(240, gl), t(6, w));
  }
  function Hn() {
    ((b = Gn(this)), t(5, b));
  }
  function ci() {
    ((w = Gn(this)), t(6, w));
  }
  function Un() {
    ((Q = Us(this.value)), t(14, Q));
  }
  function ui() {
    ((x = this.__value), t(15, x));
  }
  const Vn = () => ee.showModal();
  function fi() {
    ((x = this.__value), t(15, x));
  }
  const Kn = () => V.showModal(),
    di = () => _t.showModal();
  function Yn() {
    ((R = this.value), t(13, R));
  }
  const pi = (I) => {
    I.key === "Enter" && t(19, (te += 1));
  };
  function Wn() {
    ((H = this.checked), t(16, H), t(12, L));
  }
  const hi = () => me.showModal(),
    mi = () => We.showModal();
  function Tn() {
    ((Oe = this.value), t(56, Oe));
  }
  function gi(I) {
    dt[I ? "unshift" : "push"](() => {
      ((He = I), t(57, He));
    });
  }
  function zn() {
    ((Xe = this.value), t(22, Xe));
  }
  function Xn() {
    ((A = this.checked), t(7, A), t(8, q));
  }
  const dn = () => j.showModal();
  function Qn() {
    ((q = this.checked), t(8, q));
  }
  const Nl = () => F.showModal();
  function Es() {
    ((k = this.checked), t(10, k));
  }
  const Ns = () => ut.showModal();
  function Ps() {
    ((ue = Us(this.value)), t(20, ue));
  }
  const Ls = () => {
      t(35, (he = !0));
    },
    xn = () => bt.showModal();
  function pn() {
    ((_e = Us(this.value)), t(21, _e));
  }
  const Jn = () => Je.showModal(),
    B = () => De.showModal();
  function ye() {
    ((K = this.value), t(31, K));
  }
  function Zn() {
    ((E = this.value), t(27, E));
  }
  function Sn() {
    ((P = this.value), t(28, P));
  }
  function kn() {
    ((O = this.value), t(30, O));
  }
  function Mn() {
    ((G = this.value), t(29, G));
  }
  const En = () => at.showModal();
  function Nn(I) {
    ((M = I), t(26, M));
  }
  function Pn(I) {
    ((M = I), t(26, M));
  }
  return (
    (l.$$.update = () => {
      (l.$$.dirty[0] & 256 && t(8, q),
        l.$$.dirty[0] & 384 && q && !A && t(7, (A = !0)),
        l.$$.dirty[0] & 2048 && t(11, (D = D || "")),
        l.$$.dirty[0] & 2 && t(0, (i = f.seq)),
        l.$$.dirty[0] & 2 && (u = f.frag),
        l.$$.dirty[0] & 1048576 && t(20, ue),
        l.$$.dirty[0] & 2097152 && t(21, _e),
        l.$$.dirty[0] & 1 && xl(i),
        l.$$.dirty[0] & 8192 && Gl(),
        l.$$.dirty[0] & 32 && t(4, (v = qt(b))),
        l.$$.dirty[0] & 64 && t(4, (v = gl(w))),
        l.$$.dirty[0] & 2 && t(98, (s = f.isCircular)),
        l.$$.dirty[0] & 4096 && L !== "codon" && t(16, (H = !1)),
        (l.$$.dirty[0] & 7994781) | (l.$$.dirty[3] & 32) &&
          t(
            9,
            ({ primers: T, warnings: J } = el(te, L, R, Xe, i, d, p, v, A, Q, D, x, H, q, _e, ue, s, k)),
            T,
            (t(95, J),
            t(235, el),
            t(19, te),
            t(12, L),
            t(13, R),
            t(22, Xe),
            t(0, i),
            t(2, d),
            t(3, p),
            t(4, v),
            t(7, A),
            t(14, Q),
            t(11, D),
            t(15, x),
            t(16, H),
            t(8, q),
            t(21, _e),
            t(20, ue),
            t(98, s),
            t(10, k),
            t(1, f),
            t(239, qt),
            t(5, b),
            t(240, gl),
            t(6, w)),
          ),
        l.$$.dirty[0] & 1 && Qt(),
        l.$$.dirty[0] & 8388608 && t(23, Ze),
        l.$$.dirty[0] & 16777216 && t(24, bl),
        l.$$.dirty[0] & 12416 && t(17, (U = ct.nebuilderBlocker(L, R, A))),
        l.$$.dirty[0] & 196608 && t(18, (le = H && !U)),
        l.$$.dirty[0] & 262656 &&
          t(
            25,
            ({ primers: N, warnings: ie } = os(T, le, kc)),
            N,
            (t(96, ie),
            t(236, os),
            t(9, T),
            t(18, le),
            t(235, el),
            t(19, te),
            t(12, L),
            t(13, R),
            t(22, Xe),
            t(0, i),
            t(2, d),
            t(3, p),
            t(4, v),
            t(7, A),
            t(14, Q),
            t(11, D),
            t(15, x),
            t(16, H),
            t(8, q),
            t(21, _e),
            t(20, ue),
            t(98, s),
            t(10, k),
            t(17, U),
            t(1, f),
            t(239, qt),
            t(5, b),
            t(240, gl),
            t(6, w)),
          ),
        l.$$.dirty[3] & 12 && t(62, (n = J.concat(ie))),
        l.$$.dirty[3] & 16 && t(97, a));
    }),
    [
      i,
      f,
      d,
      p,
      v,
      b,
      w,
      A,
      q,
      T,
      k,
      D,
      L,
      R,
      Q,
      x,
      H,
      U,
      le,
      te,
      ue,
      _e,
      Xe,
      Ze,
      bl,
      N,
      M,
      E,
      P,
      G,
      O,
      K,
      z,
      pe,
      Y,
      he,
      $,
      fe,
      we,
      j,
      F,
      ee,
      V,
      me,
      _t,
      Je,
      bt,
      ut,
      it,
      Be,
      We,
      Ie,
      Ce,
      De,
      at,
      Ve,
      Oe,
      He,
      lt,
      _l,
      Rt,
      Lt,
      n,
      de,
      ne,
      $t,
      nl,
      ht,
      tt,
      pl,
      Kl,
      Mt,
      hl,
      Et,
      bs,
      qs,
      ws,
      Ft,
      Yl,
      Ol,
      Fl,
      Kt,
      ys,
      Jl,
      as,
      Pt,
      Ht,
      Zl,
      cs,
      Qt,
      As,
      Ut,
      It,
      Ks,
      ol,
      J,
      ie,
      a,
      s,
      Fs,
      wl,
      ln,
      $l,
      fs,
      Rs,
      es,
      Cs,
      ds,
      yl,
      un,
      Vl,
      Ys,
      ts,
      Ts,
      Is,
      ps,
      Al,
      Xs,
      sl,
      al,
      Ss,
      Cl,
      ks,
      Ds,
      hs,
      Qs,
      ms,
      tn,
      xs,
      Ms,
      sn,
      Os,
      js,
      Yt,
      cl,
      ls,
      nn,
      gs,
      rn,
      Bs,
      Hs,
      Js,
      fn,
      Zs,
      $s,
      ul,
      Tl,
      El,
      Ae,
      ot,
      on,
      ii,
      On,
      ri,
      jn,
      oi,
      Bn,
      ai,
      Hn,
      ci,
      Un,
      ui,
      Vn,
      fi,
      Kn,
      di,
      Yn,
      pi,
      Wn,
      hi,
      mi,
      Tn,
      gi,
      zn,
      Xn,
      dn,
      Qn,
      Nl,
      Es,
      Ns,
      Ps,
      Ls,
      xn,
      pn,
      Jn,
      B,
      ye,
      Zn,
      Sn,
      kn,
      Mn,
      En,
      Nn,
      Pn,
    ]
  );
}
class Kr extends Dl {
  constructor(e) {
    (super(), Il(this, e, Ec, Sc, Ll, {}, null, [-1, -1, -1, -1, -1, -1, -1, -1, -1]));
  }
}
function Nc(l) {
  let e;
  return {
    c() {
      ((e = c("div")),
        (e.innerHTML = `<div><div><a href="#/">Home Page</a></div> 
    
    <h3 class="">About this application</h3> 

    <p>NEBaseChanger is a simple yet powerful tool to design primers for site-directed mutagenesis experiments using the Q5 Site-Directed Mutagenesis Kit. In addition to generating primer sequences, an annealing temperature (Ta) for the PCR is calculated with knowledge of the preferred PCR polymerase, its buffer composition, the preferred primer concentration and any mismatches or loops within the primer.</p> 

	<p>The Q5 Site-Directed Mutagenesis Kit enables rapid, site-specific mutagenesis of double-stranded plasmid DNA in less than 2 hours. The kit utilizes the robust Q5 Hot Start High-Fidelity DNA Polymerase along with custom mutagenic primers to create insertions, deletions, and substitutions in a wide variety of plasmids. NEBaseChanger will design the custom mutagenic primers for use in the Q5 Site-Directed Mutagenesis Kit. Primers designed for this kit can accommodate more complex mutagenic strategies than other commercial SDM kits including multiple substitutions within a 100 nucleotide region or insertions up to 100 nucleotides long. In addition, due to the use of back-to-back primers, there is no limit to the size of the deletion that can be introduced using this kit.</p> 

    <h4 class="">System Requirements</h4> 

    <p>This application is best used on modern web browsers that are compliant with HTML5 and CSS3 standards.  Javascript must be enabled for the tool to work.</p> 


    <h4 class="orange">Updates</h4> 
    <dl><dt class="svelte-12gs2qw">v2.11.0 October 10, 2026</dt><dd><ul><li>Changing the ORF start no longer clears the mutations you&#39;ve entered; any that no longer match the new reading frame are skipped with a warning.</li> 
                <li>Typing in the sequence box is faster on long sequences.</li> 
                <li>A sample vector that fails to load now shows an error and leaves your sequence unchanged.</li> 
                <li>Pressing Enter in the mutations box builds primers; switching modes keeps a minimum primer length you set.</li> 
                <li>Help buttons, dialogs and all controls work from the keyboard, with visible focus and labels for screen readers.</li> 
                <li>Long help dialogs fit the window and keep their Close button in view; the page can be zoomed freely.</li> 
                <li>Fixed searching at either end of a partial sequence, and &quot;undefined&quot; appearing when a coordinate was cleared.</li></ul> 
        </dd><dt class="svelte-12gs2qw">v2.10.0 October 9, 2026</dt><dd><ul><li>Work (sequence, mutations, settings and primers) is now kept when you visit Help, About, Resources or Troubleshooting and come back.</li> 
                <li>Repaired Help page links and bookmarks</li> 
                <li>Fixed issue where clicking a codon after returning from another page no longer opened the substitution dialog.</li></ul> 
        </dd><dt class="svelte-12gs2qw">v2.9.0 October 8, 2026</dt><dd>Accuracy and reliability update.
            <ul><li>Fixed the primer table sometimes freezing (showing primers for an earlier entry) after an invalid character was typed; invalid input now shows a warning instead.</li> 
                <li>Fixed primers and sequence downloads for mutations near either end of partial (linear) sequences and across the origin of circular sequences.</li> 
                <li>Every primer pair is now checked against the intended mutant sequence before it is shown or downloaded.</li> 
                <li>Corrected melting temperatures (TA/AT and A\xB7C mismatch nearest-neighbor values) and degenerate base handling (IUPAC B and D codes, S/W complements).</li> 
                <li>Primers are named the same way in the table and every download (<i>ID</i>_<i>n</i>_F / <i>ID</i>_<i>n</i>_R), and lowercase marks the mutation in every download.</li> 
                <li>NEBuilder primers: duplicate sites and short fragments across the origin are now flagged, and the table order no longer changes when the option is switched on.</li> 
                <li>Downloads report problems instead of failing silently; FASTA files are wrapped at 70 characters.</li></ul> 
        </dd><dt class="svelte-12gs2qw">v2.8.4 July 10 2026</dt><dd>Repaired file downloading issue when mutation was a deletion..
        </dd><dt class="svelte-12gs2qw">v2.8.3 June 23, 2026</dt><dd>Added Q5-SDM troubleshooting module.
        </dd><dt class="svelte-12gs2qw">v2.7.3 March 2, 2026</dt><dd>Minor edit to Help to clarify when the NEBuilder option is accessible.
        </dd><dt class="svelte-12gs2qw">v2.7.2 March 2, 2026</dt><dd>Repaired reverse primer using partial sequence input, where the rev primer 5&#39; end was off by 1 nt.
        </dd><dt class="svelte-12gs2qw">v2.7.1 February 17, 2026</dt><dd>Repaired misplaced reverse primer when using partial sequence input.
        </dd><dt class="svelte-12gs2qw">v2.7.0 February 9, 2026</dt><dd>Added link to NEBuilder Calculator when NEBuilder primers are generated.
        </dd><dt class="svelte-12gs2qw">v2.6.10 February 6, 2026</dt><dd>Repaired broken Save Summary function when NEBuilder primers were generated.
        </dd><dt class="svelte-12gs2qw">v2.6.9 December 12, 2025</dt><dd>Bundled release includes following feature enhancements:
            <ul><li>support for partial template sequences.</li> 
                <li>support for merged mutations in Codon mode.</li> 
                <li>Legacy mode to emulate NEBaseChanger V1.</li> 
                <li>text selection of target sequences.</li> 
                <li>point-and-click codon selection in codon mode.</li> 
                <li>optional explicit triplet specification for point mutation designation.</li> 
                <li>ORF start reset, ORF auto location, and ORF start set by click in sequence.</li> 
                <li>Added warning that text selection fails in Safari with long sequences.</li> 
                <li>Repaired problems with circular sequence support where mutations/primers span origin.</li> 
                <li>Removed the single-site option from NEBuilder HiFi primer generation.</li> 
                <li>Repaired result sequence download and added download of all result sequences.</li> 
                <li>Updated Help to reflect the new feature set and other changes.</li></ul> 
        </dd><dt class="svelte-12gs2qw">v2.5.4 July 15, 2025</dt><dd>Revised shortcut icon (favicon).
        </dd><dt class="svelte-12gs2qw">v2.5.3 June 11, 2025</dt><dd>Removed extraneous underscore from primer ID. Removed link to NEBaseChanger v1.
        </dd><dt class="svelte-12gs2qw">v2.5.2 October 14, 2024</dt><dd>Updated Help text.
        </dd><dt class="svelte-12gs2qw">v2.5.1 July 5, 2024</dt><dd>New NEB logo.
        </dd><dt class="svelte-12gs2qw">v2.5.0 April 16, 2024</dt><dd>Added option to adjust primer minimum Tm and minimum length.
        </dd><dt class="svelte-12gs2qw">v2.4.4 February 15, 2024</dt><dd>Fixed critical issue that prevented download of the final mutated sequence in AA mode.
        </dd><dt class="svelte-12gs2qw">v2.4.3 January 24, 2024</dt><dd>Fixed annoying issue that inserted underscores after every letter in a Primer Set ID.
        </dd><dt class="svelte-12gs2qw">v2.4.2 October 17, 2023</dt><dd>Added option to preserve degenerate base codes in primers.
        </dd><dt class="svelte-12gs2qw">v2.4.1 September 28, 2023</dt><dd>Fixed issue where minimum primer length was still falling below 15 when mutations were confined to 5<sup>\u2032</sup> primer tails.
        </dd><dt class="svelte-12gs2qw">v2.4.0 September 27, 2023</dt><dd>Fixed issue where minimum primer length was falling below 15 when mutations were confined to 5<sup>\u2032</sup> primer tails.
            Changed definition of &quot;X&quot; amino acid to represent the codon NNK (was NNN).
        </dd><dt class="svelte-12gs2qw">v2.3.2 September 7, 2023</dt><dd>Changed default selection for NEBuilder Primer type to &#39;multiple&#39;.
        </dd><dt class="svelte-12gs2qw">v2.3.1 August 4, 2023</dt><dd>Raised minimum primer length to 15 nt.
        </dd><dt class="svelte-12gs2qw">v2.3.0 July 17, 2023</dt><dd>Added &#39;All&#39; option to AA dropdown -- equivalent to &#39;NNN&#39;. And added help dialog for one-letter codes. Enforced exclusivity 
            between NEBuilder HiFi primer generation and confining mutations to 5<sup>\u2032</sup> primer tails.
        </dd><dt class="svelte-12gs2qw">v2.2.1 June 27, 2023</dt><dd>Minor changes to UI and documentation.
        </dd><dt class="svelte-12gs2qw">v2.2.0 June 26, 2023</dt><dd>Added NEBuider HiFi-compatible primer generation for AA point mutation mode.
    	</dd><dt class="svelte-12gs2qw">v2.1.1 June 7, 2023</dt><dd>Fixed/restored missing Gln to AA replacement list.
    	</dd><dt class="svelte-12gs2qw">v2.1.0 June 1, 2023</dt><dd>Added codon selection strategies for AA point mutation mode.
    	</dd><dt class="svelte-12gs2qw">v2.0.1 May 23, 2023</dt><dd>Repair OneTrust cookie manager implementation.
    	</dd><dt class="svelte-12gs2qw">v2.0.0 May 17, 2023</dt><dd>Initial release of NEBaseChanger V2. <br/>
            This major version upgrade adds:<br/> 
            <ul><li>consolidated, streamlined insertion/deletion/substitution input view</li> 
            	<li>added new AA point mutation mode</li> 
            	<li>added new batch input mode allowing multiple general indel/substitution actions along with AA point mutations</li> 
            	<li>added support for ambiguous bases in replacement sequences which generate multiple primer sets</li> 
            	<li>added support for choosing between inline mutations (embedded in the middle of the primer annealing segment) and mutations confined to the 5\u2032 primer tail</li></ul></dd></dl></div>`),
        r(e, "class", "container"));
    },
    m(t, s) {
      g(t, e, s);
    },
    p: qe,
    i: qe,
    o: qe,
    d(t) {
      t && m(e);
    },
  };
}
class Pc extends Dl {
  constructor(e) {
    (super(), Il(this, e, null, Nc, Ll, {}));
  }
}
const Lc = "/assets/image003.3a95346d.jpg",
  bi = "/assets/NEBtools_help_icon.fd8dd3fc.svg",
  Gc = "/assets/check_mark.7ca807e2.svg",
  Fc = "/assets/question_mark.8bf863ee.svg";
function Yr(l, e, t) {
  const s = l.slice();
  return ((s[12] = e[t]), s);
}
function Rc(l) {
  let e,
    t,
    s,
    n,
    a = l[1].output + "";
  return {
    c() {
      ((e = c("img")),
        (s = S()),
        (n = c("p")),
        r(e, "class", "infoicon svelte-15hjavs"),
        mn(e.src, (t = Gc)) || r(e, "src", t),
        r(e, "alt", ""),
        r(n, "class", "bot-text-t svelte-15hjavs"));
    },
    m(i, u) {
      (g(i, e, u), g(i, s, u), g(i, n, u), (n.innerHTML = a));
    },
    p(i, u) {
      u & 2 && a !== (a = i[1].output + "") && (n.innerHTML = a);
    },
    d(i) {
      (i && m(e), i && m(s), i && m(n));
    },
  };
}
function Ic(l) {
  let e,
    t,
    s,
    n,
    a = l[1].output + "";
  return {
    c() {
      ((e = c("img")),
        (s = S()),
        (n = c("p")),
        r(e, "class", "infoicon svelte-15hjavs"),
        mn(e.src, (t = Fc)) || r(e, "src", t),
        r(e, "alt", ""),
        r(n, "class", "bot-text-q svelte-15hjavs"));
    },
    m(i, u) {
      (g(i, e, u), g(i, s, u), g(i, n, u), (n.innerHTML = a));
    },
    p(i, u) {
      u & 2 && a !== (a = i[1].output + "") && (n.innerHTML = a);
    },
    d(i) {
      (i && m(e), i && m(s), i && m(n));
    },
  };
}
function Dc(l) {
  let e;
  return {
    c() {
      ((e = c("div")),
        (e.innerHTML =
          'If you need additional help, please contact Technical Support by emailing info@neb.com or using the <a href="https://www.neb.com/support/neb-technical-support" target="_page">Technical Support form</a>.'),
        r(e, "class", "terminal-badge svelte-15hjavs"));
    },
    m(t, s) {
      g(t, e, s);
    },
    p: qe,
    d(t) {
      t && m(e);
    },
  };
}
function Oc(l) {
  let e,
    t = l[2],
    s = [];
  for (let n = 0; n < t.length; n += 1) s[n] = Wr(Yr(l, t, n));
  return {
    c() {
      for (let n = 0; n < s.length; n += 1) s[n].c();
      e = is();
    },
    m(n, a) {
      for (let i = 0; i < s.length; i += 1) s[i] && s[i].m(n, a);
      g(n, e, a);
    },
    p(n, a) {
      if (a & 20) {
        t = n[2];
        let i;
        for (i = 0; i < t.length; i += 1) {
          const u = Yr(n, t, i);
          s[i] ? s[i].p(u, a) : ((s[i] = Wr(u)), s[i].c(), s[i].m(e.parentNode, e));
        }
        for (; i < s.length; i += 1) s[i].d(1);
        s.length = t.length;
      }
    },
    d(n) {
      (cn(s, n), n && m(e));
    },
  };
}
function Wr(l) {
  let e,
    t,
    s,
    n = l[12] + "",
    a,
    i,
    u,
    f;
  function h(...d) {
    return l[9](l[12], ...d);
  }
  return {
    c() {
      ((e = c("button")),
        (t = c("span")),
        (t.textContent = "\u2794"),
        (s = C(" \xA0")),
        (a = C(n)),
        (i = S()),
        r(t, "class", "larger svelte-15hjavs"),
        r(e, "type", "button"),
        r(e, "class", "btn-option svelte-15hjavs"));
    },
    m(d, p) {
      (g(d, e, p), o(e, t), o(e, s), o(e, a), o(e, i), u || ((f = ce(e, "click", h)), (u = !0)));
    },
    p(d, p) {
      ((l = d), p & 4 && n !== (n = l[12] + "") && Z(a, n));
    },
    d(d) {
      (d && m(e), (u = !1), f());
    },
  };
}
function jc(l) {
  let e;
  return {
    c() {
      ((e = c("span")), (e.textContent = "\xA0\xA0\xA0\xA0\xA0"));
    },
    m(t, s) {
      g(t, e, s);
    },
    p: qe,
    d(t) {
      t && m(e);
    },
  };
}
function Bc(l) {
  let e, t, s, n, a, i;
  return {
    c() {
      ((e = c("button")),
        (t = c("span")),
        (t.textContent = "\u2B05\uFE0E"),
        (s = C(" Back")),
        r(t, "class", "larger svelte-15hjavs"),
        r(e, "class", "button btn btn-link-flat svelte-15hjavs"),
        (e.disabled = n = l[0].length === 0));
    },
    m(u, f) {
      (g(u, e, f), o(e, t), o(e, s), a || ((i = ce(e, "click", l[10])), (a = !0)));
    },
    p(u, f) {
      f & 1 && n !== (n = u[0].length === 0) && (e.disabled = n);
    },
    d(u) {
      (u && m(e), (a = !1), i());
    },
  };
}
function Hc(l) {
  let e, t, s, n, a, i, u, f, h, d, p, v, _, b, w, y, A, q, T;
  function N(L, R) {
    return L[1].isTerminal ? Rc : Ic;
  }
  let M = N(l),
    k = M(l);
  function E(L, R) {
    return L[1].isTerminal ? Dc : Oc;
  }
  let P = E(l),
    G = P(l);
  function O(L, R) {
    return L[0].length > 0 ? Bc : jc;
  }
  let K = O(l),
    D = K(l);
  return {
    c() {
      ((e = c("div")),
        (t = c("div")),
        (t.innerHTML = '<a href="#/">Home Page</a>'),
        (s = S()),
        (n = c("blockquote")),
        (n.textContent =
          "Site-directed mutagenesis (SDM) can be a challenging experiment even in the hands of experienced users. Even if you have used the same template sequence, but now incorporate new mutations, success is not guaranteed. In our experience troubleshooting with our customers over the years, most SDM issues arise from the PCR step or the competent cells used. This troubleshooting flow chart is intended to help enable you with information about corrective actions to overcome undesired results."),
        (a = S()),
        (i = c("section")),
        (u = c("h2")),
        (u.textContent = `${l[3].meta.name}`),
        (f = S()),
        (h = c("div")),
        k.c(),
        (d = S()),
        (p = c("div")),
        G.c(),
        (v = S()),
        (_ = c("hr")),
        (b = S()),
        (w = c("div")),
        D.c(),
        (y = S()),
        (A = c("button")),
        (A.textContent = "Restart"),
        r(u, "class", "svelte-15hjavs"),
        r(h, "class", "chat-bubble svelte-15hjavs"),
        r(p, "class", "actions svelte-15hjavs"),
        r(A, "class", "button btn btn-link-cta svelte-15hjavs"),
        r(w, "class", "navigation-controls svelte-15hjavs"),
        r(i, "class", "appsection svelte-15hjavs"),
        r(e, "class", "container"));
    },
    m(L, R) {
      (g(L, e, R),
        o(e, t),
        o(e, s),
        o(e, n),
        o(e, a),
        o(e, i),
        o(i, u),
        o(i, f),
        o(i, h),
        k.m(h, null),
        o(i, d),
        o(i, p),
        G.m(p, null),
        o(i, v),
        o(i, _),
        o(i, b),
        o(i, w),
        D.m(w, null),
        o(w, y),
        o(w, A),
        q || ((T = ce(A, "click", l[11])), (q = !0)));
    },
    p(L, [R]) {
      (M === (M = N(L)) && k ? k.p(L, R) : (k.d(1), (k = M(L)), k && (k.c(), k.m(h, null))),
        P === (P = E(L)) && G ? G.p(L, R) : (G.d(1), (G = P(L)), G && (G.c(), G.m(p, null))),
        K === (K = O(L)) && D ? D.p(L, R) : (D.d(1), (D = K(L)), D && (D.c(), D.m(w, y))));
    },
    i: qe,
    o: qe,
    d(L) {
      (L && m(e), k.d(), G.d(), D.d(), (q = !1), T());
    },
  };
}
function Uc(l, e, t) {
  let s, n;
  const a = {
    meta: { name: "Q5-SDM Troubleshooting Guide", initialState: "S1", version: "0.10" },
    states: {
      S1: { output: "Did you observe colonies after the transformation?", isTerminal: !1, transitions: { Yes: "S8", No: "S2" } },
      S8: {
        output: "Are you observing wildtype colonies (template plasmid sequence) or colonies with the wrong mutations?",
        isTerminal: !1,
        transitions: {
          "Wildtype colonies (colonies have the template plasmid sequence)": "S10",
          "Colonies with the wrong mutations": "S11",
        },
      },
      S10: {
        output:
          "Colonies may possess the wildtype sequence if the template plasmid from the PCR step was not fully digested by the KLD Enzyme Mix prior to transformation. <br><ul><li>Use less template (\u2264 10 ng) in the PCR step to increase the likelihood of complete digestion during the KLD reaction. </li><li>Give the KLD reaction more time to digest the template by incubating at room temperature for 30-60 minutes. </li></ul>",
        isTerminal: !0,
        transitions: {},
      },
      S11: {
        output:
          "<b>SNPs or other small sequence changes</b> usually arise through random mutations that accumulate during PCR, random mutations introduced by the competent cell, or are sequencing artifacts introduced by a DNA polymerase with lower fidelity than Q5. <br><ul><li>If the same SNP is observed consistently, sequence the plasmid template to ensure it has the sequence that you expect. </li><li>If the same SNP is not observed consistently for the same colony, it is likely a sequencing artifact. </li></ul><br><b>Nucleotide errors at 5\u2019 primer junctions</b> are most commonly due to long primer impurities that arise during oligonucleotide synthesis.  <br><ul><li> Primers purified by standard desalting does not remove N-1, N-2 impurities. We recommend HPLC purification of longer primers. </li></ul><br><b>Large indels or unexpected mutations</b> typically result from the mutagenized construct causing cellular toxicity or nonspecific PCR products. <br><ul><li> To determine if these mutations arise from PCR or transformation, you can sequence the PCR product and the transformed colony. </li><li> Optimize PCR </li><li> Try NEB\xAE Stable Competent E. coli (High Efficiency) (C3040), which are ideal for isolation of plasmid clones containing repeat elements and unstable inserts, such as retroviral/lentiviral clones. </li></ul>",
        isTerminal: !0,
        transitions: {},
      },
      S2: { output: "Have you run the PCR product on a gel?", is_terminal: !1, transitions: { Yes: "S4", No: "S3" } },
      S3: {
        output:
          "Run the PCR product on a gel to confirm a single, robust band at the expected size. We recommend performing the provided SDM Control reaction in parallel to determine if the issue is related to the Q5\xAE Hot Start High-Fidelity 2X Master Mix, input, or primers (<a href='https://www.neb.com/en-us/protocols/protocol-for-control-reaction-e0554'target='_page'>view protocol</a>). <br> If you need assistance troubleshooting your PCR, please contact Technical Support and provide a picture of your gel.",
        isTerminal: !0,
        transitions: {},
      },
      S4: {
        output: "Which option below describes the results of your gel?",
        isTerminal: !1,
        transitions: {
          "No amplification": "S6",
          "Non-specific amplification including the  desired band at expected length": "S5",
          "Non-specific amplification without the desired band at expected length": "S7",
        },
      },
      S6: {
        output:
          "Perform the Protocol for the Control Reaction (<a href='https://www.neb.com/en-us/protocols/protocol-for-control-reaction-e0554'target='_page'>view protocol</a>) and run the PCR product on an agarose gel to confirm that a 6.7kb band is observed to determine that the Q5\xAE Hot Start High-Fidelity 2X Master Mix is functional. <br><br> PCR of long templates will be extremely difficult if the template plasmid does not have integrity. Check that your template plasmid is mostly supercoiled and free from nucleases by running some of it on a gel. Confirm the sequence.",
        isTerminal: !0,
        transitions: {},
      },
      S5: {
        output:
          "Carefully extract the desired band from the gel and purify using the Monarch\xAE Spin DNA Gel Extraction Kit (T1120). Up to 4ul of gel-purified product can be used in the KLD reaction.",
        isTerminal: !0,
        transitions: {},
      },
      S7: {
        output:
          "Troubleshoot or optimize the PCR for your unique construct. <br><ul><li>Perform an annealing gradient. </li><li> Check out our <a href='https://www.neb.com/en-us/tools-and-resources/troubleshooting-guides/pcr-troubleshooting-guide' target='_page'>PCR Troubleshooting Guide</a> and <a href='https://www.neb.com/en-us/tools-and-resources/usage-guidelines/guidelines-for-pcr-optimization-with-thermophilic-dna-polymerases' target='_page'>Guidelines for PCR Optimization</a>.</li><li> Redesign and test new primer(s).</li></ul>",
        isTerminal: !0,
        transitions: {},
      },
    },
  };
  let i = a.meta.initialState,
    u = [],
    { onStateChange: f = (w) => {} } = e;
  function h(w) {
    (t(0, (u = [...u, i])), t(8, (i = s.transitions[w])));
  }
  function d() {
    if (u.length > 0) {
      const w = u[u.length - 1];
      (t(0, (u = u.slice(0, -1))), t(8, (i = w)));
    }
  }
  function p() {
    (t(8, (i = a.meta.initialState)), t(0, (u = [])));
  }
  const v = (w, y) => {
      (h(w), y.currentTarget.blur());
    },
    _ = (w) => {
      (d(), w.target.blur());
    },
    b = (w) => {
      (p(), w.target.blur());
    };
  return (
    (l.$$set = (w) => {
      "onStateChange" in w && t(7, (f = w.onStateChange));
    }),
    (l.$$.update = () => {
      (l.$$.dirty & 256 && t(1, (s = a.states[i])),
        l.$$.dirty & 2 && t(2, (n = Object.keys(s.transitions))),
        l.$$.dirty & 387 && i && (f == null || f({ stateId: i, state: s, isTerminal: s.isTerminal, historyStack: u })));
    }),
    [u, s, n, a, h, d, p, f, i, v, _, b]
  );
}
class Vc extends Dl {
  constructor(e) {
    (super(), Il(this, e, Uc, Hc, Ll, { onStateChange: 7 }));
  }
}
function zr(l, e, t) {
  const s = l.slice();
  return ((s[14] = e[t]), s);
}
function Kc(l) {
  let e,
    t,
    s,
    n = [],
    a = new Map(),
    i = l[0];
  const u = (f) => f[14].id;
  for (let f = 0; f < i.length; f += 1) {
    let h = zr(l, i, f),
      d = u(h);
    a.set(d, (n[f] = Xr(d, h)));
  }
  return {
    c() {
      ((e = c("p")), (e.textContent = "On this page"), (t = S()), (s = c("ul")));
      for (let f = 0; f < n.length; f += 1) n[f].c();
      (r(e, "class", "toc-title svelte-7ti5nw"), r(s, "class", "svelte-7ti5nw"));
    },
    m(f, h) {
      (g(f, e, h), g(f, t, h), g(f, s, h));
      for (let d = 0; d < n.length; d += 1) n[d] && n[d].m(s, null);
    },
    p(f, h) {
      h & 7 && ((i = f[0]), (n = Io(n, h, u, 1, f, i, a, s, Ro, Xr, null, zr)));
    },
    d(f) {
      (f && m(e), f && m(t), f && m(s));
      for (let h = 0; h < n.length; h += 1) n[h].d();
    },
  };
}
function Yc(l) {
  let e;
  return {
    c() {
      ((e = c("p")), (e.textContent = "No headings found."), r(e, "class", "toc-empty svelte-7ti5nw"));
    },
    m(t, s) {
      g(t, e, s);
    },
    p: qe,
    d(t) {
      t && m(e);
    },
  };
}
function Xr(l, e) {
  let t,
    s,
    n = e[14].text + "",
    a,
    i,
    u,
    f,
    h,
    d;
  function p(...v) {
    return e[10](e[14], ...v);
  }
  return {
    key: l,
    first: null,
    c() {
      ((t = c("li")),
        (s = c("a")),
        (a = C(n)),
        (u = S()),
        r(s, "href", (i = "#" + e[14].id)),
        r(s, "class", "svelte-7ti5nw"),
        r(t, "class", (f = "level-" + e[14].level + " svelte-7ti5nw")),
        ti(t, "active", e[1] === e[14].id),
        (this.first = t));
    },
    m(v, _) {
      (g(v, t, _), o(t, s), o(s, a), o(t, u), h || ((d = ce(s, "click", p)), (h = !0)));
    },
    p(v, _) {
      ((e = v),
        _ & 1 && n !== (n = e[14].text + "") && Z(a, n),
        _ & 1 && i !== (i = "#" + e[14].id) && r(s, "href", i),
        _ & 1 && f !== (f = "level-" + e[14].level + " svelte-7ti5nw") && r(t, "class", f),
        _ & 3 && ti(t, "active", e[1] === e[14].id));
    },
    d(v) {
      (v && m(t), (h = !1), d());
    },
  };
}
function Wc(l) {
  let e;
  function t(a, i) {
    return a[0].length === 0 ? Yc : Kc;
  }
  let s = t(l),
    n = s(l);
  return {
    c() {
      ((e = c("nav")), n.c(), r(e, "class", "toc svelte-7ti5nw"), r(e, "aria-label", "Table of contents"));
    },
    m(a, i) {
      (g(a, e, i), n.m(e, null));
    },
    p(a, [i]) {
      s === (s = t(a)) && n ? n.p(a, i) : (n.d(1), (n = s(a)), n && (n.c(), n.m(e, null)));
    },
    i: qe,
    o: qe,
    d(a) {
      (a && m(e), n.d());
    },
  };
}
function zc(l) {
  return l
    .toLowerCase()
    .trim()
    .replace(/[^\w\s-]/g, "")
    .replace(/\s+/g, "-")
    .replace(/-+/g, "-");
}
function Xc(l, e, t) {
  let { contentSelector: s = "main" } = e,
    { headingSelector: n = "h2, h3" } = e,
    { scrollOffset: a = 0 } = e,
    { smooth: i = !0 } = e,
    { trigger: u = 0 } = e,
    f = [],
    h = "",
    d,
    p = !1;
  function v() {
    const A = document.querySelector(s);
    if (!A) return [];
    const q = A.querySelectorAll(n),
      T = new Set();
    return Array.from(q).map((N) => {
      if (!N.id) {
        let M = zc(N.textContent || ""),
          k = M,
          E = 2;
        for (; T.has(k) || document.getElementById(k); ) k = `${M}-${E++}`;
        N.id = k;
      }
      return (
        T.add(N.id),
        a > 0 && (N.style.scrollMarginTop = `${a}px`),
        { id: N.id, text: N.textContent, level: parseInt(N.tagName[1], 10) }
      );
    });
  }
  function _() {
    (d && d.disconnect(),
      (d = new IntersectionObserver(
        (A) => {
          const q = A.filter((T) => T.isIntersecting).sort((T, N) => T.boundingClientRect.top - N.boundingClientRect.top);
          q.length > 0 && t(1, (h = q[0].target.id));
        },
        { rootMargin: `-${a}px 0px -70% 0px`, threshold: 0 },
      )),
      f.forEach((A) => {
        const q = document.getElementById(A.id);
        q && d.observe(q);
      }));
  }
  async function b() {
    (await lo(), t(0, (f = v())), _());
  }
  function w(A, q) {
    const T = document.getElementById(q);
    !T || (A.preventDefault(), T.scrollIntoView({ behavior: i ? "smooth" : "auto", block: "start" }), t(1, (h = q)));
  }
  (Zr(() => {
    if ((t(9, (p = !0)), t(0, (f = v())), _(), window.location.hash)) {
      const A = window.location.hash.slice(1),
        q = document.getElementById(A);
      q &&
        setTimeout(() => {
          (q.scrollIntoView({ behavior: "auto", block: "start" }), t(1, (h = A)));
        }, 50);
    }
  }),
    Dn(() => {
      d && d.disconnect();
    }));
  const y = (A, q) => w(q, A.id);
  return (
    (l.$$set = (A) => {
      ("contentSelector" in A && t(3, (s = A.contentSelector)),
        "headingSelector" in A && t(4, (n = A.headingSelector)),
        "scrollOffset" in A && t(5, (a = A.scrollOffset)),
        "smooth" in A && t(6, (i = A.smooth)),
        "trigger" in A && t(7, (u = A.trigger)));
    }),
    (l.$$.update = () => {
      l.$$.dirty & 640 && p && u !== void 0 && b();
    }),
    [f, h, w, s, n, a, i, u, b, p, y]
  );
}
class Qc extends Dl {
  constructor(e) {
    (super(), Il(this, e, Xc, Wc, Ll, { contentSelector: 3, headingSelector: 4, scrollOffset: 5, smooth: 6, trigger: 7, refresh: 8 }));
  }
  get refresh() {
    return this.$$.ctx[8];
  }
}
function xc(l) {
  let e,
    t,
    s,
    n,
    a,
    i,
    u,
    f,
    h,
    d,
    p,
    v,
    _,
    b,
    w,
    y,
    A,
    q,
    T,
    N,
    M,
    k,
    E,
    P,
    G,
    O,
    K,
    D,
    L,
    R,
    Q,
    ae,
    W,
    x,
    H,
    U,
    le,
    J,
    ie,
    z,
    te,
    pe,
    Y,
    ue,
    he,
    de,
    _e,
    ne,
    $,
    fe,
    we,
    j,
    F,
    ee,
    V,
    me,
    _t,
    Je,
    bt,
    ut,
    it,
    Be,
    We,
    Ie,
    Ce,
    De,
    at,
    Ve,
    $t,
    rt,
    Xt,
    Xe,
    Oe,
    He,
    Ke,
    ke,
    Ue,
    nl,
    At,
    Gt,
    pt,
    Ee,
    Qe,
    ht,
    tt,
    pl,
    Kl,
    Mt,
    hl,
    Et,
    bs,
    kl,
    qs,
    Bt,
    ws,
    Vt,
    Ft,
    Yl,
    Ol,
    Wl,
    ml,
    jl,
    Gl,
    Nt,
    zl,
    rl,
    rs,
    el,
    os,
    tl,
    qt,
    gl,
    Fl,
    Xl,
    Ze,
    vl,
    Ql,
    xl,
    lt,
    st,
    Ml,
    Kt,
    ys,
    Jl,
    as,
    Pt,
    Ht,
    Zl,
    cs,
    Qt,
    As,
    _l,
    Rt,
    Lt,
    Bl,
    bl,
    Ut,
    il,
    ll,
    ql,
    Hl,
    Ul,
    us,
    It,
    Ks,
    zs,
    ol,
    Gs,
    xt,
    tn,
    Fs,
    wl,
    ln,
    $l,
    fs,
    Rs,
    es,
    Cs,
    ds,
    yl,
    un,
    Vl,
    Ys,
    ts,
    Ts,
    Is,
    ps,
    Al,
    Xs,
    sl,
    al,
    Ss,
    Cl,
    ks,
    Ds,
    hs,
    Qs,
    ms,
    xs,
    Ms,
    sn,
    Os,
    js,
    Yt,
    cl,
    ls,
    nn,
    gs,
    rn,
    Bs,
    Hs,
    Js,
    fn,
    Zs,
    $s,
    ul,
    Tl,
    El = { contentSelector: "main", headingSelector: "h2, h3, h4", scrollOffset: 80, trigger: l[1] };
  return (
    (u = new Qc({ props: El })),
    l[2](u),
    {
      c() {
        ((e = c("div")),
          (t = c("div")),
          (t.innerHTML = '<a href="#/">Home Page</a>'),
          (s = S()),
          (n = c("br")),
          (a = c("br")),
          (i = S()),
          Re(u.$$.fragment),
          (f = S()),
          (h = c("h3")),
          (h.textContent = "BASIC WORKFLOW"),
          (d = S()),
          (p = c("p")),
          (p.textContent = "The core workflow for using NEBaseChanger is:"),
          (v = S()),
          (_ = c("ul")),
          (_.innerHTML = `<li>enter the target DNA sequence,</li>  
	<li>define the range of the mutation region,</li> 
	<li>specify the mutation (deletion/insertion/substitution),</li>  
	<li>and view/save the results.</li>`),
          (b = S()),
          (w = c("p")),
          (w.textContent =
            'Specification of the mutation target and action varies with the specific application mode: "AA Point Mutation", "Indel/Substitution", "Batch", or "Legacy". Each mode is intended to add convenience for the task at hand, but most operations can be specified in any selected mode. AA Point mutation mode is most convenient if the mutagenesis is limited to one or more single codons, whereas the general Indel/Substitution mode works with larger (and smaller) single insertions and substitutions and also allows deletions. Batch mode is designed for a longer list of mutations and allows mixing of point and indel styles of entry.'),
          (y = S()),
          (A = c("p")),
          (A.textContent = `Legacy mode is intended to emulate NEBaseChanger V1 and produce results that are consistent with the parameters used in V1. Principally, the minimum primer length is reset to 18 nt and insertions are always in the 5\u2032 tails.  Legacy mode has a simpler interface that will be
	familiar to users of V1, but it is limited in functionality.`),
          (q = S()),
          (T = c("a")),
          (T.textContent = "\u2191 Back to top"),
          (N = S()),
          (M = c("h3")),
          (M.textContent = "SEQUENCE ENTRY"),
          (k = S()),
          (E = c("p")),
          (E.textContent =
            'Enter a DNA sequence by copying and pasting it into the sequence text area or uploading a file from your computer. Acceptable formats for the DNA sequence are raw (plain text without IDs and descriptions), FASTA (first line consisting of ">ID optional description"), or GenBank. For convenience, a list of common vectors is provided, accessible from the dropdown menus above the sequence entry area.  Selecting the "Use Reverse Complement" checkbox instructs NEBaseChanger to use the reverse complement of the sequence entered in the sequence text area. Selecting the "Partial" checkbox tells NEBaseChanger that it should not assume a complete circular sequence, allowing the entry of a much shorter sequence.  However, partial sequences will need to include sufficient sequence flanking the mutation site, currently set at a minimum of 35 nt 5\u2032 and 25 nt 3\u2032. Mutations will be prohibited from being too close to the ends of the provided partial sequence fragment and cannot span the ends of the fragment.'),
          (P = S()),
          (G = c("a")),
          (G.textContent = "\u2191 Back to top"),
          (O = S()),
          (K = c("h3")),
          (K.textContent = "APPLICATION MODES"),
          (D = S()),
          (L = c("h4")),
          (L.textContent = "Point Mutation Mode"),
          (R = S()),
          (Q = c("p")),
          (Q.textContent = `AA Point Mutation mode is intended to facilitate the specification of one or more point mutations (single codon changes). 
	This mode only supports sequence replacements of exactly 3 bases (1 codon), and it does not support deletions or insertions.
	Multiple single point mutations can be entered at one time. A Merge mutations option is also available to combine adjacent point mutations in a single primer. If multiple point mutations are specified, the "Generate primers for NEBuilder HiFi" option is posible, where NEBaseChanger will generate primer pairs compatible with the NEBuilder HiFi DNA Assembly Master Mix to simplify the execution of multiple
	site-directed mutagenesis in a single reaction. The option is only displayed when multiple mutations are specified in point mutation mode.`),
          (ae = S()),
          (W = c("p")),
          (W.textContent =
            "First, enter the start position of the ORF, either directly by entering the coordinate or by clicking in the DNA sequence row of the codon view. Setting the ORF start position will highlight the ORF. ORF start position must be a whole number within the sequence; an invalid value shows a warning instead of building primers. Use the Next and Prev buttons to locate and highlight the next or previous ORF. Changing the ORF start keeps the mutations you've entered; any whose amino acid no longer matches the new reading frame is skipped, with a warning, until you edit it or move the ORF start back."),
          (x = S()),
          (H = c("p")),
          (H.textContent = `Next, enter one or more mutations of the form [old AA][codon #][new AA],
	for example, M2A, F32H, C45Y, etc., separated by spaces, commas or semicolons. The codon # is relative to the selected 
	ORF start.  Optionally, specify the exact triplet replacement by appending 
	a colon followed by the triplet, for example, M2G:GGG, A34F:TTC. Exact triplets, 
	will override automatic selection.`),
          (U = S()),
          (le = c("p")),
          (le.textContent = `Clicking on an amino acid in the codon view will initiate a dialog to select the replacement
	amino acid, including the exact triplet, and will add it to the list of mutations.
	The old AA identity is checked, but processing is not halted if the old AA does not match
	the indicated codon #. The codon number takes presidence.`),
          (J = S()),
          (ie = c("p")),
          (ie.innerHTML =
            "Select a codon selection strategy, either maximum codon usage or maximum parsimony.  For the former, codon usage is obtained from <i>E. coli K12</i>. For maximum parsimony, a simple sequence distance is calculated as the required number of base changes to go from the original codon to the replacement codon, and the triplet is chosen to minimize the changes."),
          (z = S()),
          (te = c("p")),
          (te.textContent = `Normally, for substitutions like AA point mutations, where the target region encompasses just 3 nucleotides, the replacement sequence 
	will be placed in the forward primer with 10 or more bases of annealing sequence flanking the inserted bases. If the "Confine mutations to primer 5\u2032 tails" option is selected, the replacement sequence will always be attached to the 5\u2032 end of the primers. The "5\u2032 tails" option is incompatible with NEBuilder HiFi primer option (see below).`),
          (pe = S()),
          (Y = c("p")),
          (Y.textContent = `If multiple mutations are entered and cluster together closely (ideally adjacent), the "merge mutations" option can be selected. This option combines point mutations to keep them within a single primer pair, when possible. Mutations are entered normally in the form [old AA][codon #][new AA]:[triplet], 
	for example, M1A, D3H, C4F:TTT, etc., separated by spaces, commas or semicolons.
	If the mutations are close together, generally not separated by more than 6 bases, 
	the mutations are merged and treated as a single sequence replacement operation. 
	In the example above, all 3 mutations would be embedded in a single primer pair.
	This option is currently only compatible with mutations confined to the 5\u2032 primer tail.`),
          (ue = S()),
          (he = c("p")),
          (he.innerHTML = `If the &quot;Generate primers for NEBuilder HiFi&quot; option is selected, NEBaseChanger will generate primer pairs compatible with
	the NEBuilder HiFi DNA Assembly Master Mix to simplify the execution of multiple
	site-directed mutagenesis in a single reaction. 
	NEBuilder HiFi primers cannot be generated if mutations are confined to the 5<sup>\u2032</sup> tails, and the option will not be displayed, nor is it compatible with merged mutations. Individual mutations need to be sufficiently spaced apart
	to avoid overlap clashes between adjacent forward and reverse primers.`),
          (de = S()),
          (_e = c("p")),
          (_e.innerHTML =
            'Details on using NEBuilder HiFi for multiple site mutations are available <a href="https://www.neb.com/-/media/nebus/files/application-notes/appnote_improved_methods_for_sdm_using_nebuilder_hifi_dna_assembly_master_mix.pdf?rev=918a4c52177641cbb8e2087a1286b9f4" target="_blank">here</a>.'),
          (ne = S()),
          ($ = c("h4")),
          ($.textContent = "Indel/Substitution Mutation Mode"),
          (fe = S()),
          (we = c("p")),
          (we.textContent = `Indel/Substitution mode is a general mode that facilitates the specification of mutations of arbitrary length on the input sequence.
	Substitution and insertion sequences of up to 100 bp are allowed.
`),
          (j = c("p")),
          (j.innerHTML = `Insert/delete/substitute (indel) operations require a 5&#39; flanking position (start), a
  	3&#39; flanking position (end), and a replacement sequence. The target sequence to be 
  	replaced is <b>between</b> the start and end positions. First, enter the start and end coordinates.
  	Optionally, click and drag (text select) over the target sequence in either of the DNA sequence rows.
  	The target and flanking bases will be highlighted and the start/end coordinates filled in.
  	For insertions, clicking between two bases will select the proper flanking bases.`),
          (F = S()),
          (ee = c("p")),
          (ee.textContent = `To delete a sequence, leave the replacement sequence blank.
  	To insert a sequence, the start and end should be consecutive positions. 
  	The replacement sequence will be inserted between them.`),
          (V = S()),
          (me = c("p")),
          (me.innerHTML = `For example, if start/end is 234/239 and the replacement sequence is ACGT, 
  	the operation will substitute the sequence between bases 234 and 239 with ACGT.
  	This release of NEBaseChanger will design primers 
  	accommodating inserts up to a maximum of <strong>100</strong> nucleotides.
  	Note that multiple mutations can be included within a single
  	replacement sequence as long as the sequence uses only non-ambiguous base codes. For example, 
  	if two amino acids need to be changed using a single primer pair, the start/end is
  	set to flank the targeted codons, and the replacement sequence (designed manually) contains 
  	the required substitutions at each codon.`),
          (_t = S()),
          (Je = c("p")),
          (Je.textContent = `Ambiguous base codes can be entered in the replacement sequence (N,Y,S,W, etc) 
  	and will be expanded combinatorialy over multiple primers. 
  	For example, a replacement sequence containing NS will be expanded to 8 sequences 
  	containing: AC, CC, GC, TC, AG, CG, GG, and TG, generating 8 primer pairs.`),
          (bt = S()),
          (ut = c("p")),
          (ut.textContent =
            'If the "Confine mutations to primer 5\u2032 tails" option is selected, the replacement sequence will always be attached to the 5\u2032 end of the primers, regardless of the length of the substitution. This option is incompatible with NEBuilder HiFi primer option (see below).'),
          (it = S()),
          (Be = c("p")),
          (Be.textContent = `Normally, for substitutions, if the target region is longer than 6 nucleotides, 
	the substitution sequence will be split between the two primers, attached 
	to the 5\u2032 end of the primers as non-annealing tails. The 5\u2032 ends 
	of the annealing portions of the primers align with the edges of the target region. 
	If the target region encompasses 6 or fewer nucleotides, the replacement sequence 
	will be placed in the forward primer with 10 or more bases of annealing sequence 
	flanking the inserted bases.`),
          (We = S()),
          (Ie = c("a")),
          (Ie.textContent = "\u2191 Back to top"),
          (Ce = S()),
          (De = c("h4")),
          (De.textContent = "Batch Mutation Mode"),
          (at = S()),
          (Ve = c("p")),
          (Ve.textContent = `Batch input mode allows multiple command line style operations to be applied
	to the currently loaded sequence. It is intended to facilitate the entry
	of large numbers of mutations on a single input sequence.`),
          ($t = S()),
          (rt = c("p")),
          (rt.textContent = "Batch commands can be loaded from a text file, pasted into the text area, or typed in directly."),
          (Xt = S()),
          (Xe = c("p")),
          (Xe.textContent = `All batch input operates on the currently loaded sequence. Each batch command 
	should be on a separate line. The following commands are supported in Batch mode:
	`),
          (Oe = c("ul")),
          (Oe.innerHTML = `<li>ORF: [start]</li> 
		<li>[old AA][codon #][new AA]:[triplet]</li> 
		<li>[ID] [start] [end] [replacement]</li>`),
          (He = S()),
          (Ke = c("dt")),
          (Ke.textContent = "ORF: [start]"),
          (ke = S()),
          (Ue = c("dd")),
          (Ue.innerHTML = `Specifies the coordinate of the first base of the ORF. Required for subsequent AA point mutations. <br/> 
        <br/>
		Examples:<br/>
		ORF: 1256<br/>
		ORF: 2<br/> 
		<br/>
		[start] must be a positive number in the range 1 to sequence length. <br/>
		All subsequent point mutation commands will be relative to the specified ORF start.<br/> 
		<br/><br/>`),
          (nl = S()),
          (At = c("dt")),
          (At.textContent = "[old AA][codon #][new AA]:[triplet]"),
          (Gt = S()),
          (pt = c("dd")),
          (pt.innerHTML = `Specifies an AA point mutation at a specific codon.<br/> 
		<br/>
		Examples:<br/>
		A2F<br/>
		A2F:TTT<br/> 
		<br/>
		[Old AA] and [new AA] are single letter codes for the current and mutated AA, respectively.<br/> 
		[codon #] counts triplets from the last specified ORF start coordinate. <br/>
		The :[triplet] specifier is optional. If provided, it should consist of <br/>
		a colon followed immediately by the triplet replacement sequence. If not specified,<br/> 
		(leave off the colon) the replacement triplet will be the codon with the highest usage <br/>
		in <i>E. coli</i> K12. Exact triplets, will override automatic selection.<br/> 
		<br/><br/>`),
          (Ee = S()),
          (Qe = c("dt")),
          (Qe.textContent = "[ID] [start] [end] [replacement]"),
          (ht = S()),
          (tt = c("dd")),
          (tt.innerHTML = `Specifies a general indel/substitution operation. <br/> 
		<br/>
	    Examples:<br/>
		ID1 234 239 <i>deletes</i> the sequence <i>between</i> bases 234 and 239. <br/>
		ID2 234 239 ACGT <i>substitutes</i> ACGT for the sequence <i>between</i> bases 234 and 239. <br/>
		ID3 234 235 ACGT <i>inserts</i> ACGT <i>between</i> bases 234 and 235. <br/> 
		<br/>
		[Start] and [end] are the base positions <i>flanking</i> the target sequence <br/>
		to be replaced. If [replacement] is omitted, the target sequence will be deleted.<br/>  
		To insert a sequence, [start] and [end] should be consecutive positions.<br/> 
	    <br/><br/>`),
          (pl = S()),
          (Kl = c("p")),
          (Kl.textContent = `Any ambiguous base codes (N,Y,S,W, etc) will be expanded combinatorialy 
		unless "Preserve degenerate bases" is checked.
		For example, a replacement sequence containing NS will be expanded to 8 sequences 
		containing: AC, CC, GC, TC, AG, CG, GG, and TG, generating 8 primer pairs.`),
          (Mt = S()),
          (hl = c("p")),
          (hl.textContent = `The "Preserve degenerate bases" and "Confine mutations to primer 5\u2032 tails" options are honored in Batch mode. 
		Settings for minimum primer Tm and minimum primer length are also honored.
	`),
          (Et = c("p")),
          (Et.textContent = "The following is an example of a valid set of batch commands:"),
          (bs = S()),
          (kl = c("blockquote")),
          (kl.innerHTML = `ORF: 100<br/>
		M1Y<br/>
		A42S<br/>
		P24F:TTT<br/>
		setA 200 205 NNA<br/>
		InSetB 400 401 AACGGTGTN<br/>
		Site1 100 105 ACGT<br/>
		D1 1000 1200 <br/>
		setE 1800 200 ATNCG<br/>`),
          (qs = S()),
          (Bt = c("a")),
          (Bt.textContent = "\u2191 Back to top"),
          (ws = S()),
          (Vt = c("h4")),
          (Vt.textContent = "Legacy (V1) mode"),
          (Ft = S()),
          (Yl = c("p")),
          (Yl.textContent = `Legacy mode is intended to emulate NEBaseChanger V1 and produce results 
	that are consistent with the parameters used in V1. Principally, the minimum 
	primer length is reset to 18nt and insertions are always in the 5\u2032 tails.`),
          (Ol = S()),
          (Wl = c("p")),
          (Wl.innerHTML = `Insert/delete/substitute operations require a 5&#39; <i>flanking</i> position (start),
	3&#39; <i>flanking</i> position (end); the target sequence to be mutatated 
	is <b>between</b> the start and end positions. This is a change fropm V1 where substitutions and deletions
	were identified by the ends of the target itself, not the flanking bases. 
	For deletions, leave the replacement sequence blank.`),
          (ml = S()),
          (jl = c("p")),
          (jl.textContent = `In addition to manualy setting the start/end coordinates of the target,
	the target sequence may be selected by click and drag (text selection) in the DNA sequence row. 
	For insertions, clicking between two bases will select the proper flanking bases. The target sequence
	may also be located using the search field.`),
          (Gl = S()),
          (Nt = c("p")),
          (Nt.textContent = `Ambiguous base codes can be entered in the replacement sequence (N,Y,S,W, etc) 
	and will be expanded combinatorialy. 
	For example, a replacement sequence containing NS will be expanded to 8 sequences
	containing: AC, CC, GC, TC, AG, CG, GG, and TG, generating 8 primer pairs.`),
          (zl = S()),
          (rl = c("a")),
          (rl.textContent = "\u2191 Back to top"),
          (rs = S()),
          (el = c("h3")),
          (el.textContent = "PRIMER GENERATION STRATEGY"),
          (os = S()),
          (tl = c("p")),
          (tl.textContent =
            "NEBaseChanger will design the custom mutagenic back-to-back primers for use in the Q5 Site-Directed Mutagenesis Kit. Primers designed for this kit can accommodate more complex mutagenic strategies than other commercial SDM kits including multiple substitutions within a 100 nucleotide region or insertions up to 100 nucleotides long. In addition, due to the use of back-to-back primers, there is no limit to the size of the deletion that can be introduced using this kit."),
          (qt = S()),
          (gl = c("p")),
          (gl.textContent =
            "For deletions, the 5\u2032 ends of the primers align with the edges of the deletion (the bases flanking the deleted sequence."),
          (Fl = S()),
          (Xl = c("p")),
          (Xl.textContent =
            "For insertions, the inserted sequence is attached to the 5\u2032 end of the primers as non-annealing tails. If 6 or fewer nucleotides are to be inserted, they will be placed at the 5' end of one primer. If more than 6 nucleotides are to be inserted, they will be split between the two primers to keep the primer lengths and Tm\u2019s similar."),
          (Ze = S()),
          (vl = c("p")),
          (vl.textContent =
            'For substitutions, if the target region is longer than 6 nucleotides, the substitution sequence will be split between the two primers, attached to the 5\u2032 end of the primers as non-annealing tails. The 5\u2032 ends of the annealing portions of the primers align with the edges of the target region. If the target region encompasses 6 or fewer nucleotides, the replacement sequence will be placed in the forward primer with 10 or more bases of annealing sequence flanking the inserted bases. If the "Confine mutations to primer 5\u2032 tails" option is selected, the replacement sequence will always be attached to the 5\u2032 end of the primers, regardless of the length of the substitution.'),
          (Ql = S()),
          (xl = c("p")),
          (xl.textContent = `There are two optional parameters that can be used to influence primer length: minimum primer length and minimum primer Tm with default values of 15nt and 55\xB0C, respectively. Increasing the value of either parameter will generally result in longer primers with higher specificity. To match the
	specifications used in NEBaseChanger v1, set the minimum length to 18 nt and kepp the minimum Tm at 55\xB0C. When modifications are constrained to the 5\u2032 tails, the minimum primer length only applies to the annealing portion of the primer. If a tighter range of Tm values across a larger set of primers is desired, raising the minimum Tm value may help.`),
          (lt = S()),
          (st = c("a")),
          (st.textContent = "\u2191 Back to top"),
          (Ml = S()),
          (Kt = c("h3")),
          (Kt.textContent = "RESULTS"),
          (ys = S()),
          (Jl = c("h4")),
          (Jl.textContent = "Primer Output"),
          (as = S()),
          (Pt = c("p")),
          (Pt.textContent = "The output from NEBaseChanger can be viewed in multiple ways."),
          (Ht = S()),
          (Zl = c("h4")),
          (Zl.textContent = "Required Primers Table"),
          (cs = S()),
          (Qt = c("p")),
          (Qt.textContent =
            "Displayed near the bottom of the page. In addition to the primer sequence, the length, calculated melting temperature (Tm) and suggested annealing temperature (Ta) are displayed. The primer display is automatically updated whenever the mutation region or desired sequence is changed. Forward and reverse primers are color coded and the 3-frame coding sequence is shown allowing the user to verify that the desired mutation(s) has(have) been incorporated."),
          (As = S()),
          (_l = c("h4")),
          (_l.textContent = "Downloadable formats"),
          (Rt = S()),
          (Lt = c("p")),
          (Lt.innerHTML = `Every table and download names primers the same way: <i>ID</i>_<i>n</i>_F and <i>ID</i>_<i>n</i>_R, where <i>ID</i> is the mutation&#39;s name
	(for example G10W) and <i>n</i> is the pair&#39;s number in the Required Primers table. In primer sequences, lowercase bases are the mutation
	and uppercase bases anneal to the template.`),
          (Bl = S()),
          (bl = c("dl")),
          (bl.innerHTML = `<dt>Summary table</dt><dd>A tab-delimited table containing information comparable to the Primer Result table.
	</dd><dt>All primers</dt><dd>A tab-delimited plain text list of the required primers (name, sequence, note), suitable for common oligonucleotide order forms.
	</dd><dt>Selected primers</dt><dd>A tab-delimited plain text list of the required primers (name, sequence, note), suitable for common oligonucleotide order forms. 
		Limited to just the selected primer pair.
	</dd><dt>All sequence</dt><dd>All result sequences in multi-FASTA format as a local text file.
	</dd><dt>Selected sequence</dt><dd>The result sequence for the selected primer pair in FASTA format as a local text file.</dd>`),
          (Ut = S()),
          (il = c("p")),
          (il.textContent = "\xA0"),
          (ll = S()),
          (ql = c("a")),
          (ql.textContent = "\u2191 Back to top"),
          (Hl = S()),
          (Ul = c("h3")),
          (Ul.textContent = "PROTOCOL"),
          (us = S()),
          (It = c("p")),
          (It.textContent =
            "The full Q5 Site-Directed Mutagenesis Protocol can be found in the manual, which can be downloaded from: www.neb.com/E0554"),
          (Ks = S()),
          (zs = c("p")),
          (zs.textContent = "For your convenience, a quick protocol is presented here:"),
          (ol = S()),
          (Gs = c("p")),
          (xt = c("img")),
          (Fs = S()),
          (wl = c("h4")),
          (wl.textContent = "Step I: Exponential Amplification (PCR)"),
          (ln = S()),
          ($l = c("table")),
          ($l.innerHTML = `<tr><th>\xA0</th><th>25 \u03BCl RXN</th><th>FINAL CONC.</th></tr> 
	<tr><td>Q5 Hot Start High-Fidelity 2X Master Mix</td><td>12.5 \u03BCl</td><td>1X</td></tr> 
	<tr><td>10 \u03BCM Forward Primer </td><td>1.25 \u03BCl</td><td>0.5 \u03BCM</td></tr> 
	<tr><td>10 \u03BCM Reverse Primer </td><td>1.25 \u03BCl</td><td>0.5 \u03BCM</td></tr> 
	<tr><td>Template DNA (1-25 ng/ \u03BCl) </td><td>1 \u03BCl </td><td>1-25 ng</td></tr> 
	<tr><td>Nuclease-free water </td><td>9.0 \u03BCl</td><td>\xA0</td></tr>`),
          (fs = S()),
          (Rs = c("p")),
          (Rs.textContent = "Cycling Conditions:"),
          (es = S()),
          (Cs = c("table")),
          (Cs.innerHTML = `<tr><th>STEP</th><th>TEMP</th><th>TIME</th></tr> 
	<tr><td>Initial Denaturation</td><td>98 \xB0C</td><td>30 seconds</td></tr> 
	<tr><td rowspan="3">25 Cycles</td><td>98 \xB0C</td><td>10 sec</td></tr> 
	<tr><td>50-72 \xB0C*</td><td>10-30 sec</td></tr> 
	<tr><td>72 \xB0C</td><td>20-30 seconds/kb</td></tr> 
	<tr><td>Final Extension</td><td>72 \xB0C</td><td>2 minutes</td></tr> 
	<tr><td>Hold</td><td>4-10 \xB0C</td><td>\u221E</td></tr>`),
          (ds = S()),
          (yl = c("p")),
          (yl.innerHTML =
            '*<span id="prototempmsg">For mutagenic primers, please use the annealing temperature recommended by this primer design tool.</span>'),
          (un = S()),
          (Vl = c("h4")),
          (Vl.textContent = "Step II: Kinase, Ligase & DpnI (KLD) Treatment"),
          (Ys = S()),
          (ts = c("table")),
          (ts.innerHTML = `<tr><th>\xA0</th><th>10 \u03BCl RXN </th><th>FINAL CONC.</th></tr> 
	<tr><td>PCR Product </td><td>1 \u03BCl</td><td>\xA0</td></tr> 
	<tr><td>2X KLD Reaction Buffer</td><td>5 \u03BCl </td><td>1X</td></tr> 
	<tr><td>10X KLD Enzyme Mix </td><td>1 \u03BCl</td><td>1X</td></tr> 
	<tr><td>Nuclease-free water</td><td>3 \u03BCl</td><td>\xA0</td></tr>`),
          (Ts = S()),
          (Is = c("p")),
          (Is.textContent = "Mix well by pipetting gently up and down, and incubate at room temperature for 5 minutes."),
          (ps = S()),
          (Al = c("h4")),
          (Al.textContent = "Step III:  Transformation"),
          (Xs = S()),
          (sl = c("ol")),
          (sl.innerHTML = `<li>Add 5 \u03BCl of the KLD reaction mixture directly from step II to 50 \u03BCl of chemically-competent cells.</li> 
	<li>Incubate on ice for 30 minutes.</li> 
	<li>Heat shock at 42 \xB0C for 30 seconds.</li> 
	<li>Incubate on ice for 5 minutes.</li> 
	<li>Add 950 \u03BCl of SOC, gently shake at 37 \xB0C for 1 hour.</li> 
	<li>Spread 40-100 \u03BCl onto appropriate selection plate and incubate overnight at 37 \xB0C (if necessary, make a 10-100 fold dilution of recovered cells before plating to avoid a lawn of colonies).</li>`),
          (al = S()),
          (Ss = c("p")),
          (Ss.textContent = "\xA0"),
          (Cl = S()),
          (ks = c("a")),
          (ks.textContent = "\u2191 Back to top"),
          (Ds = S()),
          (hs = c("h3")),
          (hs.textContent = "POST-REACTION TROUBLESHOOTING"),
          (Qs = S()),
          (ms = c("p")),
          (ms.innerHTML =
            'If the results of your site-directed mutagenesis experiment were unsatisfactory, try the <a href="#/troubleshoot">Q5 SDM Troubleshooting guide</a> for help. The guide can be accessed through the Troubleshooting link near the top the page.'),
          (xs = S()),
          (Ms = c("a")),
          (Ms.textContent = "\u2191 Back to top"),
          (sn = S()),
          (Os = c("h3")),
          (Os.textContent = "PRIMER CALCULATIONS"),
          (js = S()),
          (Yt = c("p")),
          (Yt.textContent =
            "NEBaseChanger uses primer melting temperature (Tm) when determining how long a primer should be. For substitutions that span a mutation region greater than 6 nucleotides, the 5' end of the primer is anchored in the center of the region and extended in the 3' direction until a desired minimum Tm is achieved. For fewer than 6 substitutions, the center of the primer is anchored to the nucleotide region and extended in both the 5' and 3' directions so that there are at least 10 nucleotides that are 100% complementary on both sides of the mutation region and until a desired minimum Tm is achieved. For deletions, the primer is designed by anchoring its 5' end to the nucleotide adjacent to the selected mutation region and extending in the 3' direction until a desired minimum Tm is achieved. For insertions, the primer is designed by anchoring its 5' end to the last nucleotide in the selected mutation region and extending in the 3' direction until a desired minimum Tm is achieved. Insertions are either added to the 5' end of one primer (6 or fewer nucleotides) or both primers (more than 6 nucleotides). Tm\u2019s are calculated using nearest neighbor thermodynamics data for matched and internal mismatched bases and corrected for buffer composition [see references below]. Tms are calculated assuming the recommended buffer composition for Hot Start High-Fidelity DNA Polymerase, and the preferred primer concentration (500nM). An annealing temperature (Ta) for the PCR is calculated in addition to the Tm, optimized for Q5 DNA polymerase."),
          (cl = S()),
          (ls = c("blockquote")),
          (ls.innerHTML = `<ul><li>SantaLucia J Jr. 1998. <em>A unified view of polymer, dumbbell, and olignucleotide DNA nearest-neighbor thermodynamics.</em> Proc. Natl. Acad. Sci. USA <strong>95</strong>:1460\u201365</li> 
	<li>Allawi HT, SantaLucia J Jr. 1997. <em>Thermodynamics and NMR of internal G\u2212T misatches in DNA.</em> Biochemistry <strong>36</strong>:10581\u201394</li> 
	<li>Allawi HT, SantaLucia J Jr. 1998. <em>Nearest-neighbor thermodynamics of internal A-C mismatches in DNA: sequence dependence and pH effects.</em> Biochemistry <strong>37</strong>:9435\u201344</li> 
	<li>Allawi HT, SantaLucia J Jr. 1998. <em>Nearest-neighbor thermodynamics parameters for internal GA mismatches in DNA.</em> Biochemistry <strong>37</strong>:2170\u201379</li> 
	<li>Allawi HT, SantaLucia J Jr. 1998. <em>Thermodynamics of internal C-T mismatches in DNA.</em> Nucleic Acids Res. <strong>26</strong>:2694\u2013701</li> 
	<li>SantaLucia, J Jr., Hicks, D. 2004. <em>The thermodynamics of DNA structural motifs.</em> Annu. Rev. Biophys. Biomol. Struct. <strong>33</strong>:415\u201340.</li> 
	<li>Peyret N, Seneviratne PA, Allawi HT, SantaLucia J Jr. 1999. <em>Nearest-neighbor thermodynamics and NMR of DNA sequences with internal A-A, C-C, G-G, and T-T mismatches.</em> Biochemistry <strong>38</strong>:3468\u201377</li> 
	<li>Bommarito S, Peyret N, SantaLucia J Jr. 2000. <em>Thermodynamic parameters for DNA sequences with dangling ends.</em> Nucleic Acids Res. <strong>28</strong>:1929\u201334</li> 
	<li>Owczarzy, R, You, Y, Moreira, B, Manthey, JA, Huang, L, Behlke, MA, Walder, JA. 2004. <em>Effects of sodium ions on DNA duplex oligomers: improved predictions of melting temperatures.</em> Biochemistry <strong>43</strong>:3537-54</li></ul>`),
          (nn = S()),
          (gs = c("a")),
          (gs.textContent = "\u2191 Back to top"),
          (rn = S()),
          (Bs = c("h3")),
          (Bs.textContent = "QUESTIONS AND COMMENTS"),
          (Hs = S()),
          (Js = c("p")),
          (Js.innerHTML =
            'NEB welcomes your questions and comments on NEBaseChanger, particularly how we could improve the tool to make it more useful in your site-directed mutagenesis projects.  Please contact us via our <a target="_blank" href="https://www.neb.com/support/neb-technical-support"><b>TECH SUPPORT</b></a> or <a href="https://www.neb.com/support/customer-feedback" target="_blank"><b>FEEDBACK</b></a> forms or <a href="mailto:info@neb.com" title="NEBaseChanger Comments"><b>email</b></a> us at info@neb.com or contact us by phone in the US at (800) 632-7799 with your feedback.'),
          (fn = S()),
          (Zs = c("a")),
          (Zs.textContent = "\u2191 Back to top"),
          r(h, "id", "tgt_workflow"),
          r(p, "class", "svelte-ll35mj"),
          r(w, "class", "svelte-ll35mj"),
          r(A, "class", "svelte-ll35mj"),
          r(T, "href", "#top"),
          r(T, "class", "back-to-top svelte-ll35mj"),
          r(M, "id", "tgt_seqentry"),
          r(E, "class", "svelte-ll35mj"),
          r(G, "href", "#top"),
          r(G, "class", "back-to-top svelte-ll35mj"),
          r(K, "id", "tgt_modes"),
          r(L, "id", "tgt_codon"),
          r(Q, "class", "svelte-ll35mj"),
          r(W, "class", "svelte-ll35mj"),
          r(H, "class", "svelte-ll35mj"),
          r(le, "class", "svelte-ll35mj"),
          r(ie, "class", "svelte-ll35mj"),
          r(te, "class", "svelte-ll35mj"),
          r(Y, "class", "svelte-ll35mj"),
          r(he, "class", "svelte-ll35mj"),
          r(_e, "class", "svelte-ll35mj"),
          r($, "id", "tgt_indel"),
          r(we, "class", "svelte-ll35mj"),
          r(j, "class", "svelte-ll35mj"),
          r(ee, "class", "svelte-ll35mj"),
          r(me, "class", "svelte-ll35mj"),
          r(Je, "class", "svelte-ll35mj"),
          r(ut, "class", "svelte-ll35mj"),
          r(Be, "class", "svelte-ll35mj"),
          r(Ie, "href", "#top"),
          r(Ie, "class", "back-to-top svelte-ll35mj"),
          r(De, "id", "tgt_batch"),
          r(Ve, "class", "svelte-ll35mj"),
          r(rt, "class", "svelte-ll35mj"),
          r(Xe, "class", "svelte-ll35mj"),
          r(Kl, "class", "svelte-ll35mj"),
          r(hl, "class", "svelte-ll35mj"),
          r(Et, "class", "svelte-ll35mj"),
          r(Bt, "href", "#top"),
          r(Bt, "class", "back-to-top svelte-ll35mj"),
          r(Vt, "id", "tgt_legacy"),
          r(Yl, "class", "svelte-ll35mj"),
          r(Wl, "class", "svelte-ll35mj"),
          r(jl, "class", "svelte-ll35mj"),
          r(Nt, "class", "svelte-ll35mj"),
          r(rl, "href", "#top"),
          r(rl, "class", "back-to-top svelte-ll35mj"),
          r(el, "id", "tgt_primergen"),
          r(tl, "class", "svelte-ll35mj"),
          r(gl, "class", "svelte-ll35mj"),
          r(Xl, "class", "svelte-ll35mj"),
          r(vl, "class", "svelte-ll35mj"),
          r(xl, "class", "svelte-ll35mj"),
          r(st, "href", "#top"),
          r(st, "class", "back-to-top svelte-ll35mj"),
          r(Kt, "id", "tgt_results"),
          r(Pt, "class", "svelte-ll35mj"),
          r(Qt, "class", "svelte-ll35mj"),
          r(Lt, "class", "svelte-ll35mj"),
          r(il, "class", "svelte-ll35mj"),
          r(ql, "href", "#top"),
          r(ql, "class", "back-to-top svelte-ll35mj"),
          r(Ul, "id", "tgt_protocol"),
          r(It, "class", "svelte-ll35mj"),
          r(zs, "class", "svelte-ll35mj"),
          mn(xt.src, (tn = Lc)) || r(xt, "src", tn),
          r(xt, "alt", "img protocol"),
          r(Gs, "class", "svelte-ll35mj"),
          r($l, "class", "protocol"),
          r(Rs, "class", "svelte-ll35mj"),
          r(Cs, "class", "protocol"),
          r(yl, "class", "svelte-ll35mj"),
          r(ts, "class", "protocol"),
          r(Is, "class", "svelte-ll35mj"),
          r(Ss, "class", "svelte-ll35mj"),
          r(ks, "href", "#top"),
          r(ks, "class", "back-to-top svelte-ll35mj"),
          r(hs, "id", "tgt_troubleshoot"),
          r(ms, "class", "svelte-ll35mj"),
          r(Ms, "href", "#top"),
          r(Ms, "class", "back-to-top svelte-ll35mj"),
          r(Os, "id", "tgt_primercalcs"),
          r(Yt, "class", "svelte-ll35mj"),
          r(gs, "href", "#top"),
          r(gs, "class", "back-to-top svelte-ll35mj"),
          r(Bs, "id", "tgt_questions"),
          r(Js, "class", "svelte-ll35mj"),
          r(Zs, "href", "#top"),
          r(Zs, "class", "back-to-top svelte-ll35mj"),
          r(e, "id", "top"),
          r(e, "class", "container"));
      },
      m(Ae, ot) {
        (g(Ae, e, ot),
          o(e, t),
          o(e, s),
          o(e, n),
          o(e, a),
          o(e, i),
          Ge(u, e, null),
          o(e, f),
          o(e, h),
          o(e, d),
          o(e, p),
          o(e, v),
          o(e, _),
          o(e, b),
          o(e, w),
          o(e, y),
          o(e, A),
          o(e, q),
          o(e, T),
          o(e, N),
          o(e, M),
          o(e, k),
          o(e, E),
          o(e, P),
          o(e, G),
          o(e, O),
          o(e, K),
          o(e, D),
          o(e, L),
          o(e, R),
          o(e, Q),
          o(e, ae),
          o(e, W),
          o(e, x),
          o(e, H),
          o(e, U),
          o(e, le),
          o(e, J),
          o(e, ie),
          o(e, z),
          o(e, te),
          o(e, pe),
          o(e, Y),
          o(e, ue),
          o(e, he),
          o(e, de),
          o(e, _e),
          o(e, ne),
          o(e, $),
          o(e, fe),
          o(e, we),
          o(e, j),
          o(e, F),
          o(e, ee),
          o(e, V),
          o(e, me),
          o(e, _t),
          o(e, Je),
          o(e, bt),
          o(e, ut),
          o(e, it),
          o(e, Be),
          o(e, We),
          o(e, Ie),
          o(e, Ce),
          o(e, De),
          o(e, at),
          o(e, Ve),
          o(e, $t),
          o(e, rt),
          o(e, Xt),
          o(e, Xe),
          o(e, Oe),
          o(e, He),
          o(e, Ke),
          o(e, ke),
          o(e, Ue),
          o(e, nl),
          o(e, At),
          o(e, Gt),
          o(e, pt),
          o(e, Ee),
          o(e, Qe),
          o(e, ht),
          o(e, tt),
          o(e, pl),
          o(e, Kl),
          o(e, Mt),
          o(e, hl),
          o(e, Et),
          o(e, bs),
          o(e, kl),
          o(e, qs),
          o(e, Bt),
          o(e, ws),
          o(e, Vt),
          o(e, Ft),
          o(e, Yl),
          o(e, Ol),
          o(e, Wl),
          o(e, ml),
          o(e, jl),
          o(e, Gl),
          o(e, Nt),
          o(e, zl),
          o(e, rl),
          o(e, rs),
          o(e, el),
          o(e, os),
          o(e, tl),
          o(e, qt),
          o(e, gl),
          o(e, Fl),
          o(e, Xl),
          o(e, Ze),
          o(e, vl),
          o(e, Ql),
          o(e, xl),
          o(e, lt),
          o(e, st),
          o(e, Ml),
          o(e, Kt),
          o(e, ys),
          o(e, Jl),
          o(e, as),
          o(e, Pt),
          o(e, Ht),
          o(e, Zl),
          o(e, cs),
          o(e, Qt),
          o(e, As),
          o(e, _l),
          o(e, Rt),
          o(e, Lt),
          o(e, Bl),
          o(e, bl),
          o(e, Ut),
          o(e, il),
          o(e, ll),
          o(e, ql),
          o(e, Hl),
          o(e, Ul),
          o(e, us),
          o(e, It),
          o(e, Ks),
          o(e, zs),
          o(e, ol),
          o(e, Gs),
          o(Gs, xt),
          o(e, Fs),
          o(e, wl),
          o(e, ln),
          o(e, $l),
          o(e, fs),
          o(e, Rs),
          o(e, es),
          o(e, Cs),
          o(e, ds),
          o(e, yl),
          o(e, un),
          o(e, Vl),
          o(e, Ys),
          o(e, ts),
          o(e, Ts),
          o(e, Is),
          o(e, ps),
          o(e, Al),
          o(e, Xs),
          o(e, sl),
          o(e, al),
          o(e, Ss),
          o(e, Cl),
          o(e, ks),
          o(e, Ds),
          o(e, hs),
          o(e, Qs),
          o(e, ms),
          o(e, xs),
          o(e, Ms),
          o(e, sn),
          o(e, Os),
          o(e, js),
          o(e, Yt),
          o(e, cl),
          o(e, ls),
          o(e, nn),
          o(e, gs),
          o(e, rn),
          o(e, Bs),
          o(e, Hs),
          o(e, Js),
          o(e, fn),
          o(e, Zs),
          ($s = !0),
          ul ||
            ((Tl = [
              ce(T, "click", an),
              ce(G, "click", an),
              ce(Ie, "click", an),
              ce(Bt, "click", an),
              ce(rl, "click", an),
              ce(st, "click", an),
              ce(ql, "click", an),
              ce(ks, "click", an),
              ce(Ms, "click", an),
              ce(gs, "click", an),
              ce(Zs, "click", an),
            ]),
            (ul = !0)));
      },
      p(Ae, [ot]) {
        const on = {};
        (ot & 2 && (on.trigger = Ae[1]), u.$set(on));
      },
      i(Ae) {
        $s || (ve(u.$$.fragment, Ae), ($s = !0));
      },
      o(Ae) {
        (be(u.$$.fragment, Ae), ($s = !1));
      },
      d(Ae) {
        (Ae && m(e), l[2](null), Fe(u), (ul = !1), Zt(Tl));
      },
    }
  );
}
function an(l) {
  (l.preventDefault(), window.scrollTo({ top: 0, behavior: "smooth" }));
}
function Jc(l, e, t) {
  let s,
    n = 0;
  function a(i) {
    dt[i ? "unshift" : "push"](() => {
      ((s = i), t(0, s));
    });
  }
  return [s, n, a];
}
class Zc extends Dl {
  constructor(e) {
    (super(), Il(this, e, Jc, xc, Ll, {}));
  }
}
function $c(l) {
  let e;
  return {
    c() {
      ((e = c("div")),
        (e.innerHTML = `<div><a href="#/">Go Back</a></div> 
<h3>Additional Resources</h3> 

<h4>Products:</h4> 
	<ul><li><a target="_blank" href="https://www.neb.com/products/e0554-q5-site-directed-mutagenesis-kit#Product%20Information">Q5<sup>\xAE</sup> Site-Directed Mutagenesis Kit</a></li> 
        <li><a target="_blank" href="https://www.neb.com/products/e0552-q5-site-directed-mutagenesis-kit-without-competent-cells#Product%20Information">Q5<sup>\xAE</sup> Site-Directed Mutagenesis Kit (Without Competent Cells)</a></li> 
        <li><a target="_blank" href="https://www.neb.com/products/m0554-kld-enzyme-mix#Product%20Information">KLD Enzyme Mix</a></li></ul> 
<h4>Videos:</h4> 
    <ul><li><a target="_blank" href="https://www.neb.com/tools-and-resources/video-library/overview-of-the-q5-site-directed-mutagenesis-kit?autoplay=1">Overview of the Q5<sup>\xAE</sup> Site-Directed Mutagenesis Kit</a></li> 
        <li><a target="_blank" href="https://www.neb.com/tools-and-resources/video-library/troubleshooting-tips-for-q5-site-directed-mutagenesis-kit?autoplay=1">Troubleshooting tips for Q5<sup>\xAE</sup> Site-Directed Mutagenesis Kit</a></li> 
        <li><a target="_blank" href="https://www.neb.com/tools-and-resources/video-library/can-i-make-multiple-mutations-with-the-q5-site-directed-mutagenesis-kit?autoplay=1">Quick Tips - Can I make multiple mutations with the Q5<sup>\xAE</sup> site-directed mutagenesis kit?</a></li> 
        <li><a target="_blank" href="https://www.neb.com/tools-and-resources/video-library/quick-tips-how-can-i-improve-the-efficiency-of-the-dpni-digestion">Quick tips - How can I improve the efficiency of the DpnI digestion?</a></li></ul> 
<h4>Application Note:</h4> 
    <ul><li><a target="_blank" href="https://www.neb.com/-/media/nebus/files/application-notes/appnote_improved_methods_for_sdm_using_nebuilder_hifi_dna_assembly_master_mix">Improved Methods for Site-directed Mutagenesis using NEBuilder\xAE HiFi DNA Assembly Master Mix</a></li></ul>`),
        r(e, "class", "container"));
    },
    m(t, s) {
      g(t, e, s);
    },
    p: qe,
    i: qe,
    o: qe,
    d(t) {
      t && m(e);
    },
  };
}
function eu(l) {
  return [];
}
class tu extends Dl {
  constructor(e) {
    (super(), Il(this, e, eu, $c, Ll, {}));
  }
}
const lu = { "/resources": tu, "/about": Pc, "/help": Zc, "/troubleshoot": Vc, "/": Kr, "*": Kr },
  su = "/assets/2024_NEB_Logo_50.6f07573d.svg",
  nu = "/assets/appMainLogo.01242724.svg",
  iu = "/assets/NEBtools_about_icon.0535f502.svg";
function ru(l) {
  let e,
    t,
    s,
    n,
    a,
    i,
    u,
    f,
    h,
    d,
    p,
    v,
    _,
    b,
    w,
    y,
    A,
    q,
    T,
    N,
    M,
    k,
    E,
    P,
    G,
    O,
    K,
    D,
    L,
    R,
    Q,
    ae,
    W,
    x,
    H,
    U,
    le,
    J,
    ie,
    z,
    te,
    pe,
    Y,
    ue,
    he,
    de,
    _e,
    ne,
    $,
    fe,
    we,
    j;
  return {
    c() {
      ((e = c("div")),
        (t = c("h1")),
        (s = C(l[0])),
        (n = S()),
        (a = c("div")),
        (i = c("div")),
        (u = c("a")),
        (f = c("img")),
        (d = S()),
        (p = c("div")),
        (v = c("a")),
        (_ = c("img")),
        (w = S()),
        (y = c("div")),
        (A = S()),
        (q = c("div")),
        (T = c("div")),
        (N = c("a")),
        (M = c("img")),
        (E = S()),
        (P = c("span")),
        (P.textContent = "ABOUT"),
        (G = C(`\xA0\xA0
            `)),
        (O = c("a")),
        (K = c("img")),
        (L = S()),
        (R = c("span")),
        (R.textContent = "HELP"),
        (Q = C(`\xA0\xA0
            `)),
        (ae = c("a")),
        (W = c("img")),
        (H = S()),
        (U = c("span")),
        (U.textContent = "RESOURCES"),
        (le = C(`\xA0\xA0
            `)),
        (J = c("a")),
        (ie = c("img")),
        (te = S()),
        (pe = c("span")),
        (pe.textContent = "TROUBLESHOOT"),
        (Y = C("\xA0\xA0")),
        (ue = S()),
        (he = c("div")),
        (de = c("span")),
        (_e = C("v")),
        (ne = C(l[1])),
        ($ = S()),
        (fe = c("hr")),
        (we = S()),
        (j = c("hr")),
        r(t, "class", "visually-hidden svelte-171zae2"),
        mn(f.src, (h = su)) || r(f, "src", h),
        r(f, "alt", "New England Biolabs"),
        r(f, "class", "svelte-171zae2"),
        r(u, "href", "https://www.neb.com"),
        r(u, "class", "svelte-171zae2"),
        r(i, "id", "neb-logo"),
        r(i, "class", "svelte-171zae2"),
        r(_, "class", "tool-header-logo svelte-171zae2"),
        mn(_.src, (b = nu)) || r(_, "src", b),
        r(_, "alt", "NEBaseChanger"),
        r(v, "href", "https://nebasechanger.neb.com"),
        r(v, "class", "svelte-171zae2"),
        r(p, "id", "app-logo"),
        r(p, "class", "svelte-171zae2"),
        r(y, "class", "orange-line svelte-171zae2"),
        mn(M.src, (k = iu)) || r(M, "src", k),
        r(M, "alt", ""),
        r(M, "class", "svelte-171zae2"),
        r(P, "class", ""),
        r(N, "href", "#/about"),
        r(N, "class", "svelte-171zae2"),
        mn(K.src, (D = bi)) || r(K, "src", D),
        r(K, "alt", ""),
        r(K, "class", "svelte-171zae2"),
        r(R, "class", ""),
        r(O, "href", "#/help"),
        r(O, "class", "svelte-171zae2"),
        mn(W.src, (x = bi)) || r(W, "src", x),
        r(W, "alt", ""),
        r(W, "class", "svelte-171zae2"),
        r(U, "class", ""),
        r(ae, "href", "#/resources"),
        r(ae, "class", "svelte-171zae2"),
        mn(ie.src, (z = bi)) || r(ie, "src", z),
        r(ie, "alt", ""),
        r(ie, "class", "svelte-171zae2"),
        r(pe, "class", ""),
        r(J, "href", "#/troubleshoot"),
        r(J, "class", "svelte-171zae2"),
        r(T, "id", "header-link-container"),
        r(T, "class", "svelte-171zae2"),
        r(de, "class", ""),
        r(he, "class", "tool-header-version svelte-171zae2"),
        r(he, "id", "appVersion"),
        r(fe, "class", "clear-hr svelte-171zae2"),
        r(e, "id", "tool-header"),
        r(e, "class", "svelte-171zae2"),
        r(j, "class", "clear-hr svelte-171zae2"));
    },
    m(F, ee) {
      (g(F, e, ee),
        o(e, t),
        o(t, s),
        o(e, n),
        o(e, a),
        o(a, i),
        o(i, u),
        o(u, f),
        o(a, d),
        o(a, p),
        o(p, v),
        o(v, _),
        o(e, w),
        o(e, y),
        o(e, A),
        o(e, q),
        o(q, T),
        o(T, N),
        o(N, M),
        o(N, E),
        o(N, P),
        o(T, G),
        o(T, O),
        o(O, K),
        o(O, L),
        o(O, R),
        o(T, Q),
        o(T, ae),
        o(ae, W),
        o(ae, H),
        o(ae, U),
        o(T, le),
        o(T, J),
        o(J, ie),
        o(J, te),
        o(J, pe),
        o(T, Y),
        o(q, ue),
        o(q, he),
        o(he, de),
        o(de, _e),
        o(de, ne),
        o(q, $),
        o(q, fe),
        g(F, we, ee),
        g(F, j, ee));
    },
    p(F, [ee]) {
      (ee & 1 && Z(s, F[0]), ee & 2 && Z(ne, F[1]));
    },
    i: qe,
    o: qe,
    d(F) {
      (F && m(e), F && m(we), F && m(j));
    },
  };
}
function ou(l, e, t) {
  let { appName: s = "Anonymous" } = e,
    { version: n = "v0.0.0" } = e;
  return (
    (l.$$set = (a) => {
      ("appName" in a && t(0, (s = a.appName)), "version" in a && t(1, (n = a.version)));
    }),
    [s, n]
  );
}
class au extends Dl {
  constructor(e) {
    (super(), Il(this, e, ou, ru, Ll, { appName: 0, version: 1 }));
  }
}
function cu(l) {
  let e, t, s, n, a, i, u, f, h, d, p, v;
  return {
    c() {
      ((e = c("hr")),
        (t = S()),
        (s = c("div")),
        (n = c("div")),
        (n.innerHTML = `<a href="https://www.neb.com/support/neb-technical-support" target="" style="margin: 10px 20px;cursor: pointer;font-size: 14px;font-weight: 700;letter-spacing: .3px;line-height: 1.65; color: white; text-decoration: none;">Tech Support</a> 
        <a href="https://www.neb.com/support/customer-feedback" target="" style="margin: 10px 20px;cursor: pointer;font-size: 14px;font-weight: 700;letter-spacing: .3px;line-height: 1.65; color: white; text-decoration: none;">Feedback</a> 
        <a href="https://www.neb.com/about-neb/neb-overview" target="" style="margin: 10px 20px;cursor: pointer;font-size: 14px;font-weight: 700;letter-spacing: .3px;line-height: 1.65; color: white; text-decoration: none;">NEB Overview</a> 
        <a href="https://www.neb.com/site-map" target="" style="margin: 10px 20px;cursor: pointer;font-size: 14px;font-weight: 700;letter-spacing: .3px;line-height: 1.65; color: white; text-decoration: none;">Site Map</a>`),
        (a = S()),
        (i = c("div")),
        (i.innerHTML = `<a href="https://www.neb.com/trademarks" target="" style="margin: 10px 20px;cursor: pointer;font-size: 14px;font-weight: 700;letter-spacing: .3px;line-height: 1.65; color: white; text-decoration: none;">Trademarks</a> 
        <a href="https://www.neb.com/external-tools-legal-and-disclaimers" target="" style="margin: 10px 20px;cursor: pointer;font-size: 14px;font-weight: 700;letter-spacing: .3px;line-height: 1.65; color: white; text-decoration: none;">Legal and Disclaimers</a> 
        <a href="https://www.neb.com/privacy-policy" target="" style="margin: 10px 20px;cursor: pointer;font-size: 14px;font-weight: 700;letter-spacing: .3px;line-height: 1.65; color: white; text-decoration: none;">Privacy</a> 
        <a href="https://www.neb.com/cookie-policy" target="" style="margin: 10px 20px;cursor: pointer;font-size: 14px;font-weight: 700;letter-spacing: .3px;line-height: 1.65; color: white; text-decoration: none;">Cookie Policy</a> 
        <a href="https://www.neb.com/terms-of-use" target="" style="margin: 10px 20px;cursor: pointer;font-size: 14px;font-weight: 700;letter-spacing: .3px;line-height: 1.65; color: white; text-decoration: none;">Terms of Use</a>`),
        (u = S()),
        (f = c("div")),
        (h = c("span")),
        (d = C("\xA9 Copyright ")),
        (p = C(l[0])),
        (v = C(" New England Biolabs. All Rights Reserved.")),
        r(e, "class", "clear-hr svelte-1fj6u4d"),
        Ws(n, "margin", "15px auto"),
        Ws(n, "width", "fit-content"),
        Ws(i, "margin", "15px auto"),
        Ws(i, "width", "fit-content"),
        Ws(f, "margin", "50px 0 0 0"),
        Ws(f, "text-align", "center"),
        Ws(f, "padding-bottom", "20px"),
        Ws(f, "font-size", "smaller"),
        r(s, "id", "tool-footer"),
        Ws(s, "color", "white"),
        Ws(s, "margin", "15px 0 0 0"),
        Ws(s, "padding-top", "30px"),
        Ws(s, "background-color", "#5e5e60"),
        Ws(s, "font-family", "Arial"));
    },
    m(_, b) {
      (g(_, e, b), g(_, t, b), g(_, s, b), o(s, n), o(s, a), o(s, i), o(s, u), o(s, f), o(f, h), o(h, d), o(h, p), o(h, v));
    },
    p(_, [b]) {
      b & 1 && Z(p, _[0]);
    },
    i: qe,
    o: qe,
    d(_) {
      (_ && m(e), _ && m(t), _ && m(s));
    },
  };
}
function uu(l, e, t) {
  let { copyright: s = new Date().getFullYear() } = e;
  return (
    (l.$$set = (n) => {
      "copyright" in n && t(0, (s = n.copyright));
    }),
    [s]
  );
}
class fu extends Dl {
  constructor(e) {
    (super(), Il(this, e, uu, cu, Ll, { copyright: 0 }));
  }
}
function du(l) {
  let e, t, s, n, a, i, u;
  return (
    (t = new au({ props: { appName: "NEBaseChanger", version: l[0] } })),
    (n = new Ko({ props: { routes: lu } })),
    (i = new fu({})),
    {
      c() {
        ((e = c("main")), Re(t.$$.fragment), (s = S()), Re(n.$$.fragment), (a = S()), Re(i.$$.fragment));
      },
      m(f, h) {
        (g(f, e, h), Ge(t, e, null), o(e, s), Ge(n, e, null), o(e, a), Ge(i, e, null), (u = !0));
      },
      p: qe,
      i(f) {
        u || (ve(t.$$.fragment, f), ve(n.$$.fragment, f), ve(i.$$.fragment, f), (u = !0));
      },
      o(f) {
        (be(t.$$.fragment, f), be(n.$$.fragment, f), be(i.$$.fragment, f), (u = !1));
      },
      d(f) {
        (f && m(e), Fe(t), Fe(n), Fe(i));
      },
    }
  );
}
function pu(l) {
  return ["2.11.0"];
}
class hu extends Dl {
  constructor(e) {
    (super(), Il(this, e, pu, du, Ll, {}));
  }
}
new hu({ target: document.getElementById("app") });
