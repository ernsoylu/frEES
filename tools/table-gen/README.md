# `tools/table-gen` — Property Table Generator (`FRPHTAB1`)

Generates precomputed real-fluid property tables in `FRPHTAB1` format and validates interpolation accuracy against thermodynamic baselines.

```bash
./run.sh                                    # Water + R134a -> fixtures/proptables/
./run.sh /tmp/tables --sweep --samples 1500 # resolution ladder, writes no tables
./run.sh /tmp/tables --fluids R1234yf,CO2   # additional fluids
./run.sh --help                             # print options
```

---

## 1. Geometry & Coordinate Formulation

All units are in SI:

- **Saturation Lines**: $n_{\text{sat}}$ samples uniform in $\ln P$ over $[p_{\min}, p_{\max}]$, where $p_{\min} = \max(1.2 \cdot p_{\text{triple}}, 10^{-4} \cdot p_{\text{crit}})$ and $p_{\max} = 0.75 \cdot p_{\text{crit}}$. Interpolated using cubic Hermite splines on $\ln P$ with central-difference slopes.
- **Two-Phase Mixture Region**: $h_f(P) \le h \le h_g(P)$: exact thermodynamic mixture relations off saturation lines:
  $$T = T_{\text{sat}}(P), \quad v = v_f + x \cdot v_{fg}, \quad s = s_f + x \cdot s_{fg}, \quad x = \frac{h - h_f}{h_g - h_f}$$
- **Superheated Vapor**: Bicubic spline over $(P, y)$ with $y = h - h_g(P)$ to align coordinate axes along the saturation boundary. $P$ is log-spaced over $[p_{\min}, p_{\max}]$; $y$ is quadratically spaced over $[0, \Delta h_{\text{vapor},\max}/0.9]$ to concentrate nodes in regions of high curvature near the saturation dome.
- **Subcooled Liquid**: Bicubic spline over $(P, y)$ with normalized coordinates:
  $$y = \frac{h_f(P) - h}{h_f(P) - h_{\text{cold}}(P)} \in [0, 1], \quad h_{\text{cold}}(P) = h(P, T_{\text{low}})$$
- **Service Limits**: Lookups outside $[p_{\min}, 0.95 \cdot p_{\max}]$ or beyond maximum enthalpy ranges return out-of-bounds indications.

Tabulated outputs in plane order:
- **`T`** (Temperature in K)
- **`Dmass`** (Density in $\text{kg/m}^3$)
- **`Smass`** (Specific entropy in $\text{J/(kg}\cdot\text{K)}$)

---

## 2. On-Disk Binary Format (`FRPHTAB1`)

All integers and floating-point values are stored in little-endian byte order.

### Header Layout

| Offset | Size | Type | Field | Description |
|---:|---:|---|---|---|
| 0 | 8 | `u8[8]` | Magic | ASCII string `"FRPHTAB1"` |
| 8 | 2 | `u16` | `format_version` | Format version (= 1) |
| 10 | 1 | `u8` | `elem_kind` | Element kind: 0 = `f64`, 1 = `f32` (payload only) |
| 11 | 1 | `u8` | `flags` | Bit 0: liquid piece present; Bit 1: liquid $y$ normalized |
| 12 | 4 | `u32` | `n_sat` | Saturation-line samples |
| 16 | 4 | `u32` | `n_p` | Pressure nodes per 2D surface |
| 20 | 4 | `u32` | `n_dh` | Enthalpy depth nodes per 2D surface |
| 24 | 4 | `u32` | `n_props` | Number of properties (= 3: T, Dmass, Smass) |
| 28 | 4 | `u32` | `header_bytes` | Byte offset of payload start (8-byte aligned) |
| 32 | 8 | `f64` | `p_min` | Minimum pressure [Pa] |
| 40 | 8 | `f64` | `p_max` | Maximum pressure [Pa] |
| 48 | 8 | `f64` | `p_serve_max` | Maximum served pressure [Pa] |
| 56 | 8 | `f64` | `p_liquid_min` | Minimum liquid pressure [Pa] |
| 64 | 8 | `f64` | `dh_vapor_max` | Maximum vapor enthalpy depth [J/kg] |
| 72 | 8 | `f64` | `dh_liquid_max` | Maximum liquid enthalpy depth |
| 80 | 8 | `f64` | `h_top` | Upper enthalpy limit [J/kg] |
| 88 | 8 | `f64` | `t_low` | Lower temperature limit [K] |
| 96 | 8 | `f64` | `p_crit` | Critical pressure [Pa] |
| 104 | 8 | `f64` | `t_crit` | Critical temperature [K] |
| 112 | 8 | `f64` | `p_triple` | Triple-point pressure [Pa] |
| 120 | 8 | `f64` | `t_triple` | Triple-point temperature [K] |
| 128 | 2 | `u16` | `fluid_len` | Length of fluid name string |
| 130 | 2 | `u16` | `version_len` | Length of engine version string |
| 132 | 4 | `u32` | `backfilled_nodes` | Repaired non-finite node count |
| 136 | — | `u8[]` | Strings | Fluid name, version string, zero padding to 8-byte alignment |

### Payload Layout

1. **Saturation Block**: 9 contiguous arrays of length $n_{\text{sat}}$:
   `log_p`, `t_sat`, `h_f`, `h_g`, `v_f`, `v_fg`, `s_f`, `s_fg`, `h_cold`
2. **Vapor Surface**:
   - `p_grid` [$n_p$]
   - `y_grid` [$n_{dh}$]
   - `T`, `Dmass`, `Smass` planes [$n_p \times n_{dh}$ each] in row-major order
3. **Liquid Surface** (if enabled in flags):
   - `p_grid` [$n_p$]
   - `y_grid` [$n_{dh}$]
   - `T`, `Dmass`, `Smass` planes [$n_p \times n_{dh}$ each]
