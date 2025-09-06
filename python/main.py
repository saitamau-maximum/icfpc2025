# pip install ortools
from typing import List, Dict, Any, Tuple
from ortools.sat.python import cp_model
import json
import subprocess
import os


def de_bruijn(k: int, alphabet: List[int]) -> List[int]:
    """De Bruijn sequence for alphabet^k, returned as a cycle starting at 0...0.
    Iterative prefer-one implementation; returns length=len(alphabet)**k."""
    n = len(alphabet)
    a = [0] * (n * k)
    seq = []

    def db(t: int, p: int):
        if t > k:
            if k % p == 0:
                seq.extend(a[1 : p + 1])
        else:
            a[t] = a[t - p]
            db(t + 1, p)
            for j in range(a[t - p] + 1, n):
                a[t] = j
                db(t + 1, t)

    db(1, 1)
    return [alphabet[i] for i in seq]


def make_plan(n: int, budget_ratio_for_core: float = 0.6) -> str:
    """1回の/探索で識別力を高くする非適応プラン。de Bruijn を順/逆/回転で重ねる。
    長さは 18n を超えないように打ち切り。"""
    Lmax = 18 * n
    if Lmax <= 0:
        return ""
    base = list(range(6))
    # k: 6^k <= budget
    import math

    core_budget = max(1, int(math.floor(budget_ratio_for_core * Lmax)))
    k = 1
    while (6 ** (k + 1)) <= core_budget:
        k += 1
    B = de_bruijn(k, base)  # list of ints
    # markers to re-synchronize
    M = [0, 1, 2, 3, 4, 5] * 3  # length 18

    def rot(seq, r):
        return [((d + r) % 6) for d in seq]

    seqs = [B, list(reversed(B)), rot(B, 1), rot(B, 2), rot(B, 3)]
    plan = []
    for S in seqs:
        if len(plan) + len(M) + len(S) > Lmax:
            break
        plan += M
        plan += S
    # fallback if nothing fit (tiny n)
    if not plan:
        plan = B[:Lmax]
    return "".join(str(d) for d in plan)


def solve_from_trace(n: int, plan: str, outputs: List[int]) -> Dict[str, Any]:
    """CP-SATで T[r][a] と Label[r] を推定し、/guess 用の map を返す。
    - n: 部屋数
    - plan: '0123...' の扉列（長さ L）
    - outputs: 各時点の2bitラベル（長さ L+1）
    """
    L = len(plan)
    assert len(outputs) == L + 1, "outputs は plan より 1 長い必要があります"
    a = [int(c) for c in plan]
    assert all(0 <= d <= 5 for d in a), "扉は 0..5"
    assert all(0 <= v <= 3 for v in outputs), "ラベルは 0..3"

    model = cp_model.CpModel()

    # --- Variables ---

    # x[i][r] : 位置 i が部屋 r にいる（one-hot）
    x = [[model.NewBoolVar(f"x_{i}_{r}") for r in range(n)] for i in range(L + 1)]

    # S_int[i] : 位置 i の部屋インデックス（0..n-1）
    S_int = [model.NewIntVar(0, n - 1, f"S_{i}") for i in range(L + 1)]

    # Label[r] : 部屋 r の 2bit ラベル（0..3）
    Label = [model.NewIntVar(0, 3, f"label_{r}") for r in range(n)]

    # T[r][a] : 部屋 r から扉 a で進む先の部屋インデックス（0..n-1）
    T = [[model.NewIntVar(0, n - 1, f"T_{r}_{d}") for d in range(6)] for r in range(n)]

    # tsel[r][d][r2] : (T[r][d] == r2) を示す指示変数（出入バランスに利用）
    tsel = [
        [[model.NewBoolVar(f"tsel_{r}_{d}_{r2}") for r2 in range(n)] for d in range(6)]
        for r in range(n)
    ]

    # --- One-hot & channeling ---

    for i in range(L + 1):
        # one-hot
        model.Add(sum(x[i][r] for r in range(n)) == 1)
        # S_int[i] = sum r * x[i][r]
        # （整数=ブール加重和のチャネリング）
        model.Add(S_int[i] == sum(r * x[i][r] for r in range(n)))

    # 始点の対称性破り
    model.Add(S_int[0] == 0)
    # 弱い対称性破り: S_i <= i （初出順で番号が増える方向）
    for i in range(L + 1):
        model.Add(S_int[i] <= i if i < n else S_int[i] <= n - 1)

    # --- ラベル整合: x[i][r] => Label[r] == outputs[i] ---
    for i in range(L + 1):
        v = outputs[i]
        for r in range(n):
            model.Add(Label[r] == v).OnlyEnforceIf(x[i][r])

    # --- 遷移決定性: x[i][r] => S_{i+1} == T[r][a_i] ---
    for i in range(L):
        d = a[i]
        for r in range(n):
            model.Add(S_int[i + 1] == T[r][d]).OnlyEnforceIf(x[i][r])

    # --- 出入バランスのための T==r2 のブール化と対称制約 ---
    # tsel[r][d][r2] <=> (T[r][d] == r2)
    for r in range(n):
        for d in range(6):
            # ちょうど1つの r2 が真
            model.Add(sum(tsel[r][d][r2] for r2 in range(n)) == 1)
            # 値=ブールのチャネリング
            for r2 in range(n):
                model.Add(T[r][d] == r2).OnlyEnforceIf(tsel[r][d][r2])
                model.Add(T[r][d] != r2).OnlyEnforceIf(tsel[r][d][r2].Not())

    # 双方向の“個数一致”：任意の (r, r2) で r→r2 の本数 == r2→r の本数
    for r in range(n):
        for r2 in range(r + 1, n):
            out_r_to_r2 = sum(tsel[r][d][r2] for d in range(6))
            out_r2_to_r = sum(tsel[r2][d][r] for d in range(6))
            model.Add(out_r_to_r2 == out_r2_to_r)

    # --- セーフなドメイン縮小（色違いは同一部屋にできない） ---
    # outputs[i] != outputs[j] => S_i != S_j
    # x[i][r] + x[j][r] <= 1 を課す（ラベル不一致の位置同士は同一rに割当不可）
    buckets = {v: [i for i in range(L + 1) if outputs[i] == v] for v in range(4)}
    for v1 in range(4):
        for v2 in range(4):
            if v1 == v2:
                continue
            for i in buckets[v1]:
                for j in buckets[v2]:
                    # 同じrは許さない
                    for r in range(n):
                        model.Add(x[i][r] + x[j][r] <= 1)

    # --- ソルブ ---
    solver = cp_model.CpSolver()
    solver.parameters.max_time_in_seconds = 60.0  # 必要に応じて
    solver.parameters.num_search_workers = 8  # 並列
    # 目的関数は未設定（可行解で十分）。タイブレークしたい場合はここに追加。

    status = solver.Solve(model)
    if status not in (cp_model.OPTIMAL, cp_model.FEASIBLE):
        raise RuntimeError("No feasible model found")

    # --- 解の取り出し ---
    S = [solver.Value(S_int[i]) for i in range(L + 1)]
    labels = [solver.Value(Label[r]) for r in range(n)]
    T_mat = [[solver.Value(T[r][d]) for d in range(6)] for r in range(n)]

    # --- 扉どうしのペアリングを構成（/guess の connections 用） ---
    connections = build_connections_from_T(T_mat)

    return {
        "rooms": labels,  # 長さ n の 0..3
        "startingRoom": 0,
        "connections": connections,  # schema に合う list[dict]
    }


def build_connections_from_T(T: List[List[int]]) -> List[Dict[str, Dict[str, int]]]:
    """遷移 T[r][d]=r' を双方向の扉対応（(r,d) <-> (r',d')）に拡張して
    /guess の connections 配列を作る。必要条件は pairwise バランス：
    |{d | T[r][d]=r'}| == |{d' | T[r'][d']=r}|.
    """
    n = len(T)
    used = [[False] * 6 for _ in range(n)]
    conns: List[Tuple[Tuple[int, int], Tuple[int, int]]] = []

    # まず r != r' の間でペアリング
    for r in range(n):
        for r2 in range(r + 1, n):
            outs_r_to_r2 = [d for d in range(6) if T[r][d] == r2]
            outs_r2_to_r = [d for d in range(6) if T[r2][d] == r]
            # 同数であるはず（CP制約で保証）
            k = min(len(outs_r_to_r2), len(outs_r2_to_r))
            # 単純にジップで対応（任意だが整合）
            for d, d2 in zip(outs_r_to_r2[:k], outs_r2_to_r[:k]):
                if used[r][d] or used[r2][d2]:
                    continue
                used[r][d] = used[r2][d2] = True
                conns.append(((r, d), (r2, d2)))

    # 次に r == r の自己向き遷移（T[r][d] == r）を処理
    for r in range(n):
        self_doors = [d for d in range(6) if T[r][d] == r and not used[r][d]]
        # ここでは (d <-> d) の自己ループで埋める（常に可能）
        for d in self_doors:
            if not used[r][d]:
                used[r][d] = True
                conns.append(((r, d), (r, d)))

    # 未処理が残っていれば（理論上は残らないはずだが）同室内で適当にペアにする
    for r in range(n):
        remaining = [d for d in range(6) if not used[r][d]]
        # 残りは T[r][d] == some r2 != r のはずだが、相手側がすでに埋まっている場合がある。
        # 応急処置として自己ペアを許す（T上は自己戻りでなくても、/guessの“扉結線”は
        # ここで定義されるため、双方向性は満たす）。ただし T の逆遷移が一致しないケースでは
        # 存在しないはず（出入バランスがあれば残りは出ない）。
        for d in remaining:
            used[r][d] = True
            conns.append(((r, d), (r, d)))

    # 重複を片側だけに
    uniq = []
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


# ----------------------------
# 使い方の例（擬似）
# ----------------------------
if __name__ == "__main__":
    # n は /select した問題の部屋数（問題ページで公開）
    n = 6
    problem_name = "primus"
    plan = make_plan(n)
    print(plan)

    working_dir = os.path.dirname(os.path.abspath(__file__))
    root_dir = os.path.join(working_dir, "..")
    print(root_dir)

    subprocess.run(
        [
            os.path.join(root_dir, "target/release/aedificium"),
            "select",
        ],
        input=problem_name,
        capture_output=True,
        text=True,
    )

    resp = subprocess.run(
        [
            os.path.join(root_dir, "target/release/aedificium"),
            "explore",
        ],
        input=f'["{plan}"]',
        capture_output=True,
        text=True,
    )
    outputs = json.loads(resp.stdout)["results"][0]
    print(outputs)

    map_spec = solve_from_trace(n, plan, outputs)
    print(map_spec)

    resp = subprocess.run(
        [
            os.path.join(root_dir, "target/release/aedificium"),
            "guess",
        ],
        input=json.dumps(map_spec),
        capture_output=True,
        text=True,
    )
    print(resp.stdout)
