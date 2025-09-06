from __future__ import annotations

from typing import Dict, List, Tuple, Any, Optional
import os
import tempfile
import subprocess


class CNFBuilder:
    """DIMACS CNFビルダー。

    - 変数は1始まりの整数ID
    - 名前<->IDの双方向対応を保持
    - exactly-one は sequential AMO + at-least-one
    - at-most-one は Sinzの逐次エンコーディング
    """

    def __init__(self) -> None:
        self._next_var: int = 1
        self.name_to_id: Dict[str, int] = {}
        self.id_to_name: List[str] = [""]  # ダミー(1-index)
        self.clauses: List[List[int]] = []

    # 変数管理 --------------------------------------------------------------
    def new_var(self, name: str) -> int:
        if name in self.name_to_id:
            return self.name_to_id[name]
        vid = self._next_var
        self._next_var += 1
        self.name_to_id[name] = vid
        self.id_to_name.append(name)
        return vid

    def lit(self, name: str, is_positive: bool = True) -> int:
        vid = self.new_var(name)
        return vid if is_positive else -vid

    def num_vars(self) -> int:
        return self._next_var - 1

    # 句管理 ----------------------------------------------------------------
    def add_clause(self, lits: List[int]) -> None:
        # 空句は追加しない（UNSAT確定になるため）
        if not lits:
            # 予期しない空句にならないよう呼び出し側で回避する
            raise ValueError("empty clause")
        self.clauses.append(lits)

    def add_at_least_one(self, vars_pos: List[int]) -> None:
        if not vars_pos:
            raise ValueError("at_least_one on empty set")
        self.add_clause(list(vars_pos))

    def add_at_most_one(self, vars_pos: List[int], tag: str) -> None:
        # Sinz 2005 sequential encoding
        m = len(vars_pos)
        if m <= 1:
            return
        s_vars: List[int] = [self.new_var(f"s_amo_{tag}_{i}") for i in range(1, m)]

        # (¬x1 ∨ s1)
        self.add_clause([-vars_pos[0], s_vars[0]])

        # for i=2..m-1:
        for i in range(1, m - 1):
            xi = vars_pos[i]
            si = s_vars[i]
            si_1 = s_vars[i - 1]
            # (¬xi ∨ si)
            self.add_clause([-xi, si])
            # (¬si-1 ∨ si)
            self.add_clause([-si_1, si])
            # (¬xi ∨ ¬si-1)
            self.add_clause([-xi, -si_1])

        # (¬xm ∨ ¬s_{m-1})
        self.add_clause([-vars_pos[-1], -s_vars[-1]])

    def add_exactly_one(self, vars_pos: List[int], tag: str) -> None:
        if not vars_pos:
            raise ValueError("exactly_one on empty set")
        self.add_at_least_one(vars_pos)
        self.add_at_most_one(vars_pos, tag)

    # DIMACS出力 -----------------------------------------------------------
    def to_dimacs(self, comments: Optional[List[str]] = None) -> str:
        lines: List[str] = []
        for c in comments or []:
            lines.append(f"c {c}")
        lines.append(f"p cnf {self.num_vars()} {len(self.clauses)}")
        for cls in self.clauses:
            lines.append(" ".join(str(lit) for lit in cls) + " 0")
        return "\n".join(lines) + "\n"


def _bits2(v: int) -> Tuple[int, int]:
    return (v & 1, (v >> 1) & 1)


def encode_trace_to_cnf(
    n: int, plan: str, outputs: List[int]
) -> Tuple[CNFBuilder, Dict[str, Any]]:
    """与えられた plan と outputs から、未知の T と Label を含むSATモデルをCNF化。

    変数:
      - x[i][r]          : 位置 i で部屋 r にいる
      - t[r][d][r2]      : T[r][d] == r2 のone-hot
      - label_b[r][b]    : 部屋rのラベルbit b (b=0 or 1)

    制約:
      - 各 i で exactly-one_r x[i][r]
      - x[0][0] を真に固定
      - t[r][d][*] は exactly-one
      - x[i][r] ∧ t[r][a_i][r2] ⇒ x[i+1][r2]
      - x[i][r] ⇒ label_b[r] == outputs[i]_b （b=0,1）

    注意: ペアリングや入出次数の厳密整合はここでは課さない（後段で接続生成）
    """
    L = len(plan)
    assert len(outputs) == L + 1
    assert all(0 <= int(c) <= 5 for c in plan)
    assert all(0 <= v <= 3 for v in outputs)

    a = [int(c) for c in plan]

    cnf = CNFBuilder()

    # 変数IDを先に生成（デコード容易化）
    x_ids: List[List[int]] = [
        [cnf.new_var(f"x:{i}:{r}") for r in range(n)] for i in range(L + 1)
    ]
    t_ids: List[List[List[int]]] = [
        [[cnf.new_var(f"t:{r}:{d}:{r2}") for r2 in range(n)] for d in range(6)]
        for r in range(n)
    ]
    label_ids: List[Tuple[int, int]] = [
        (cnf.new_var(f"lb:{r}:0"), cnf.new_var(f"lb:{r}:1")) for r in range(n)
    ]

    # x: exactly-one per i
    for i in range(L + 1):
        cnf.add_exactly_one(x_ids[i], tag=f"x_{i}")

    # start room = 0
    cnf.add_clause([x_ids[0][0]])

    # t: exactly-one per (r, d)
    for r in range(n):
        for d in range(6):
            cnf.add_exactly_one(t_ids[r][d], tag=f"t_{r}_{d}")

    # pairwise balance: 任意の (r, r2) で r→r2 の本数 == r2→r の本数
    # 6x6 のペア変数 y_{d,d'} を導入し、
    #  行: A_d ⇒ 行内でちょうど1つ選ばれる(AMO + 条件付きALO)
    #  列: B_d' ⇒ 列内でちょうど1つ選ばれる(AMO + 条件付きALO)
    #  y ⇒ A, y ⇒ B を入れて整合をとる
    for r in range(n):
        for r2 in range(r + 1, n):
            A = [t_ids[r][d][r2] for d in range(6)]
            B = [t_ids[r2][d][r] for d in range(6)]
            # y matrix
            Y: List[List[int]] = [
                [cnf.new_var(f"pair:{r}:{r2}:{d}:{d2}") for d2 in range(6)]
                for d in range(6)
            ]

            # 行ごと: A_d => exactly-one(Y[d,*]) with conditional ALO and unconditional AMO
            for d in range(6):
                row = Y[d]
                # y => A_d
                for yv in row:
                    cnf.add_clause([-yv, A[d]])
                # at-most-one (unconditional)
                cnf.add_at_most_one(row, tag=f"pair_row_{r}_{r2}_{d}")
                # conditional at-least-one: (¬A_d ∨ y1 ∨ ... ∨ y6)
                cnf.add_clause([-A[d]] + row)

            # 列ごと: B_d' => exactly-one(Y[* ,d'])
            for d2 in range(6):
                col = [Y[d][d2] for d in range(6)]
                for yv in col:
                    cnf.add_clause([-yv, B[d2]])
                cnf.add_at_most_one(col, tag=f"pair_col_{r}_{r2}_{d2}")
                cnf.add_clause([-B[d2]] + col)

    # step: x[i][r] ∧ t[r][a_i][r2] ⇒ x[i+1][r2]
    for i in range(L):
        dsel = a[i]
        for r in range(n):
            x_ir = x_ids[i][r]
            for r2 in range(n):
                cnf.add_clause([-x_ir, -t_ids[r][dsel][r2], x_ids[i + 1][r2]])

    # label consistency: x[i][r] ⇒ label[r] == outputs[i]
    for i in range(L + 1):
        v = outputs[i]
        b0, b1 = _bits2(v)
        for r in range(n):
            lb0, lb1 = label_ids[r]
            cnf.add_clause([-x_ids[i][r], lb0 if b0 == 1 else -lb0])
            cnf.add_clause([-x_ids[i][r], lb1 if b1 == 1 else -lb1])

    meta = {
        "n": n,
        "L": L,
        "x_ids": x_ids,
        "t_ids": t_ids,
        "label_ids": label_ids,
    }
    return cnf, meta


def run_glucose_on_cnf(
    cnf: CNFBuilder, solver_path: Optional[str] = None
) -> Dict[int, bool]:
    """CNFを一時ファイルに書き、glucose/minisatを起動して充足割当を取得。

    - glucose: 標準出力の v 行を読む
    - minisat: 2番目の引数で結果ファイルを指定し、そこから読む
    """
    # ソルバーの場所
    if solver_path is None:
        # プロジェクト同梱のglucoseを優先
        here = os.path.dirname(os.path.abspath(__file__))
        root = os.path.abspath(os.path.join(here, ".."))
        cand = os.path.join(root, "glucose", "simp", "glucose")
        if os.path.exists(cand) and os.access(cand, os.X_OK):
            solver_path = cand
        else:
            # PATHから探す
            from shutil import which

            solver_path = which("glucose") or which("minisat")
            if solver_path is None:
                raise FileNotFoundError("glucose/minisat が見つかりません")

    with tempfile.TemporaryDirectory() as td:
        cnf_path = os.path.join(td, "problem.cnf")
        out_path = os.path.join(td, "model.out")
        with open(cnf_path, "w") as f:
            f.write(cnf.to_dimacs(["generated by sat_external.py"]))

        solver_base = os.path.basename(solver_path or "").lower()
        if "minisat" in solver_base:
            proc = subprocess.run(
                [solver_path, cnf_path, out_path], capture_output=True, text=True
            )
            stdout = proc.stdout + "\n" + proc.stderr
            if "UNSAT" in stdout.upper():
                raise RuntimeError("CNF is UNSAT")
            if not os.path.exists(out_path):
                raise RuntimeError(
                    f"minisat result file not found. Output: {stdout[:200]}"
                )
            with open(out_path, "r") as rf:
                res_text = rf.read()
            lits = parse_v_lines(res_text)
        else:
            # glucose 系は -model を付けると v 行でモデルを出力
            args = (
                [solver_path, "-model", cnf_path]
                if "glucose" in solver_base
                else [solver_path, cnf_path]
            )
            proc = subprocess.run(args, capture_output=True, text=True)
            stdout = proc.stdout + "\n" + proc.stderr
            if "UNSAT" in stdout.upper():
                raise RuntimeError("CNF is UNSAT")
            if "SAT" not in stdout.upper():
                # 一部ソルバはSAT/UNSATをstderrに出す
                # それでも見つからない場合は異常終了
                raise RuntimeError(f"Solver output not recognized: {stdout[:200]}")
            lits = parse_v_lines(stdout)

        assignment: Dict[int, bool] = {}
        for lit in lits:
            v = abs(lit)
            assignment[v] = lit > 0
        if not assignment:
            raise RuntimeError(
                "Solver reported SAT but no model literals were parsed. Try --solver minisat or glucose with -model support."
            )
        return assignment


def parse_v_lines(text: str) -> List[int]:
    """モデル行から整数リテラル列を抽出。

    - "v 1 -2 3 -4 0" 形式
    - 数値のみの行 "1 -2 3 -4 0" にも対応
    - 複数行を結合
    """
    lits: List[int] = []
    for line in text.splitlines():
        s = line.strip()
        if not s:
            continue
        if s.startswith("c ") or s.startswith("C "):
            continue
        if (
            s.startswith("s ")
            or s.upper().startswith("SAT")
            or s.upper().startswith("UNSAT")
        ):
            # ステータス行は無視
            continue
        parts = s.split()
        if parts and (parts[0] == "v" or parts[0] == "V"):
            parts = parts[1:]
        # 数値以外は捨てつつ 0 で打ち切り
        for tok in parts:
            if tok == "0":
                break
            try:
                lits.append(int(tok))
            except ValueError:
                pass
    return lits


def _assignment_get(
    assignment: Dict[int, bool], var_id: int, default: bool = False
) -> bool:
    return assignment.get(var_id, default)


def build_connections_from_T(T: List[List[int]]) -> List[Dict[str, Dict[str, int]]]:
    """main.py と同等の接続構築。T[r][d]=r' を(可能な限り)双方向ペアにする。"""
    n = len(T)
    used = [[False] * 6 for _ in range(n)]
    conns: List[Tuple[Tuple[int, int], Tuple[int, int]]] = []

    for r in range(n):
        for r2 in range(r + 1, n):
            outs_r_to_r2 = [d for d in range(6) if T[r][d] == r2]
            outs_r2_to_r = [d for d in range(6) if T[r2][d] == r]
            k = min(len(outs_r_to_r2), len(outs_r2_to_r))
            for d, d2 in zip(outs_r_to_r2[:k], outs_r2_to_r[:k]):
                if used[r][d] or used[r2][d2]:
                    continue
                used[r][d] = used[r2][d2] = True
                conns.append(((r, d), (r2, d2)))

    for r in range(n):
        self_doors = [d for d in range(6) if T[r][d] == r and not used[r][d]]
        for d in self_doors:
            if not used[r][d]:
                used[r][d] = True
                conns.append(((r, d), (r, d)))

    for r in range(n):
        remaining = [d for d in range(6) if not used[r][d]]
        for d in remaining:
            used[r][d] = True
            conns.append(((r, d), (r, d)))

    uniq: List[Dict[str, Dict[str, int]]] = []
    seen = set()
    for (r, d), (r2, d2) in conns:
        key = tuple(sorted([(r, d), (r2, d2)]))
        if key in seen:
            continue
        seen.add(key)
        uniq.append(
            {
                "from": {"room": r, "door": d},
                "to": {"room": r2, "door": d2},
            }
        )
    return uniq


def decode_model_to_map_spec(
    assignment: Dict[int, bool], meta: Dict[str, Any]
) -> Dict[str, Any]:
    n: int = meta["n"]
    L: int = meta["L"]
    x_ids: List[List[int]] = meta["x_ids"]
    t_ids: List[List[List[int]]] = meta["t_ids"]
    label_ids: List[Tuple[int, int]] = meta["label_ids"]

    # 位置ごとの部屋推定
    S: List[int] = []
    for i in range(L + 1):
        r_found = 0
        for r in range(n):
            if _assignment_get(assignment, x_ids[i][r], False):
                r_found = r
                break
        S.append(r_found)

    # ラベル
    labels: List[int] = []
    for r in range(n):
        b0, b1 = label_ids[r]
        v0 = 1 if _assignment_get(assignment, b0, False) else 0
        v1 = 1 if _assignment_get(assignment, b1, False) else 0
        labels.append(v0 | (v1 << 1))

    # T
    T: List[List[int]] = [[0] * 6 for _ in range(n)]
    for r in range(n):
        for d in range(6):
            r2_sel = 0
            for r2 in range(n):
                if _assignment_get(assignment, t_ids[r][d][r2], False):
                    r2_sel = r2
                    break
            T[r][d] = r2_sel

    # 接続へ
    connections = build_connections_from_T(T)
    return {
        "rooms": labels,
        "startingRoom": 0,
        "connections": connections,
    }


def solve_from_trace_with_sat(
    n: int, plan: str, outputs: List[int], solver_path: Optional[str] = None
) -> Dict[str, Any]:
    cnf, meta = encode_trace_to_cnf(n, plan, outputs)
    assignment = run_glucose_on_cnf(cnf, solver_path=solver_path)
    return decode_model_to_map_spec(assignment, meta)
