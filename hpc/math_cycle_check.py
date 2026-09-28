"""Exact checks for the scoped SL(2,R) x SU(2) numerical-cycle fixture.
Adapted from atlas-dispatch-s2-wt/ac_product_phi_check.py, GPL-3.0-or-later.
No historical output, orbit ranks or Rust result is used as a golden.
"""
from fractions import Fraction
import re


def mathematical_evidence(case, entry, stdout, stderr):
    """Never infer a mathematical pass from an incomplete/error execution."""
    kind = case.get("mathematical_check")
    if kind is None:
        return None
    if kind != "sl2_su2_numerical_cycles":
        raise ValueError("unknown mathematical checker")
    if entry["exit_status"] != 0 or entry["timed_out"] or stderr:
        return {"status": "NOT_RUN_EXECUTION_FAILED"}
    try:
        return {"status": "PASS_SCOPED_NUMERICAL_IDENTITIES",
                "bounds": check_product_phi(stdout.decode()),
                "general_cycle_acceptance": False,
                "cutoff_completeness_proved": False}
    except (ValueError, UnicodeError) as error:
        return {"status": "FAIL", "error": str(error)}


def integers(text):
    if not re.fullmatch(r'\[\s*(?:-?\d+\s*(?:,\s*-?\d+\s*)*)?\]', text):
        raise ValueError('malformed integer vector')
    return [int(x) for x in re.findall(r'-?\d+', text)]


def product(columns, vector, rows):
    if len(columns) != len(vector) or any(len(c) != rows for c in columns):
        raise ValueError('matrix/vector shape mismatch')
    return [sum(c[i]*v for c,v in zip(columns,vector)) for i in range(rows)]


def exact_rank(columns, rows):
    """Column rank over Q; also certifies the reported full kernel dimension."""
    pivots = {}
    for column in columns:
        if len(column) != rows:
            raise ValueError('ragged matrix')
        v = list(map(Fraction,column))
        for pivot,b in sorted(pivots.items()):
            a = v[pivot]
            if a:
                v = [x-a*y for x,y in zip(v,b)]
        pivot = next((i for i,x in enumerate(v) if x),None)
        if pivot is not None:
            a = v[pivot]
            pivots[pivot] = [x/a for x in v]
    return len(pivots)


def check_product_phi(stdout):
    lines = iter(l for l in stdout.splitlines() if l.startswith('ACF_'))
    result = []

    def take(pattern):
        line = next(lines,'')
        m = re.fullmatch(pattern,line)
        if not m:
            raise ValueError('missing, reordered or invalid product Phi record: '+line)
        return m

    for bound in (4,6):
        take(rf'ACF_BEGIN bound={bound} complex=4 real=3')
        complex_orbits = []
        for i in range(4):
            m = take(rf'ACF_COMPLEX bound={bound} orbit={i} H=(\[[^]]*\]) diagram=(\[[^]]*\]) dimension=(\d+) real_forms=(\[[^]]*\])')
            h,diagram,dim,forms = integers(m[1]),integers(m[2]),int(m[3]),integers(m[4])
            if len(h)!=2 or len(diagram)!=2 or dim!=sum(diagram):
                raise ValueError('product complex orbit coordinates/dimension mismatch')
            complex_orbits.append(dict(orbit=i,H=h,diagram=diagram,dimension=dim,real_forms=forms))
        if sorted(o['diagram'] for o in complex_orbits)!=[[0,0],[0,2],[2,0],[2,2]]:
            raise ValueError('all four component-labelled product orbits required')
        if [o['dimension'] for o in complex_orbits]!=[0,2,2,4]:
            raise ValueError('complex orbits not in dimension order')
        real = []
        for i in range(3):
            m = take(rf'ACF_REAL bound={bound} orbit={i} H=(\[[^]]*\]) x=(\d+) complex=(\d+) dimension=(\d+) pairs=(\d+)')
            h,x,ci,dim,count = integers(m[1]),*map(int,m.groups()[1:])
            if len(h)!=2 or ci not in range(4) or dim!=complex_orbits[ci]['dimension'] or x not in range(3):
                raise ValueError('real/complex orbit identity mismatch')
            real.append(dict(orbit=i,H=h,x=x,complex=ci,dimension=dim,pairs=count))
        for o in complex_orbits:
            if o['real_forms'] != [r['orbit'] for r in real if r['complex']==o['orbit']]:
                raise ValueError('real forms do not exhaust the recorded orbit identities')
        m = take(rf'ACF_MATRIX bound={bound} complex=(\d+) closure=(\[[^]]*\]) top=(\[[^]]*\]) ambient=(\d+) rows=(\d+) columns=(\d+) kernel=(\d+) boundary=(\d+)')
        ci,closure,top = int(m[1]),integers(m[2]),integers(m[3])
        ambient,rows,cols,kdim,bdim = map(int,m.groups()[3:])
        if (ci not in range(4) or complex_orbits[ci]['diagram']!=[2,0]
                or top!=complex_orbits[ci]['real_forms'] or len(top)!=2
                or closure!=[r['orbit'] for r in real if r['dimension']==0]+top
                or sorted(closure)!=[0,1,2]
                or not 0<rows<ambient<=400 or not 0<kdim<cols<=1000
                or not 0<bdim<=ambient or cols!=sum(real[i]['pairs'] for i in closure)):
            raise ValueError('wrong target component, closure or matrix shape')
        def vectors(tag,count,length):
            values = []
            for j in range(count):
                m = take(rf'ACF_{tag} bound={bound} column={j} vector=(\[[^]]*\])')
                v = integers(m[1])
                if len(v)!=length:
                    raise ValueError('wrong '+tag+' column shape')
                values.append(v)
            return values
        px = vectors('PX',ambient,rows)
        boundary = vectors('BOUNDARY',bdim,ambient)
        if any(product(px,b,rows)!=[0]*rows for b in boundary):
            raise ValueError('boundary survives projection')
        columns = []
        for oi in closure:
            for j in range(real[oi]['pairs']):
                index = len(columns)
                m = take(rf'ACF_COLUMN bound={bound} column={index} orbit={oi} pair={j} rank=(\d+) ambient=(\[[^]]*\]) quotient=(\[[^]]*\])')
                dim,a,q = int(m[1]),integers(m[2]),integers(m[3])
                if dim<1 or len(a)!=ambient or len(q)!=rows or product(px,a,rows)!=q:
                    raise ValueError('invalid inducing dimension or Q=PX*T column')
                take(rf'ACF_PHI bound={bound} column={index} inducing=.+')
                columns.append(dict(orbit=oi,pair=j,rank=dim,ambient=a,quotient=q))
        q = [c['quotient'] for c in columns]
        kernel = vectors('KERNEL',kdim,cols)
        qr = exact_rank(q,rows)
        if kdim != cols-qr or exact_rank(kernel,cols)!=kdim or any(product(q,k,rows)!=[0]*rows for k in kernel):
            raise ValueError('reported kernel is not the full independent kernel')
        rank_rows = []
        for oi in top:
            m = take(rf'ACF_RANK bound={bound} orbit={oi} row=(\[[^]]*\]) kernel_product=(\[[^]]*\])')
            row,printed = integers(m[1]),integers(m[2])
            if (row!=[c['rank'] if c['orbit']==oi else 0 for c in columns] or printed!=[0]*kdim
                    or any(sum(a*b for a,b in zip(row,k)) for k in kernel)):
                raise ValueError('orbit-tagged dimension row does not annihilate full kernel')
            rank_rows.append(row)
        bindings = [int(take(rf'ACF_BIND bound={bound} identity={i} column=(\d+)')[1])
                    for i in range(4)]
        modules = []
        for compact in range(3):
            for i,(x,lr0,weight0) in enumerate(((0,-1,1),(0,0,2),(1,-1,-1),(1,0,-2),(2,0,0))):
                prefix = rf'bound={bound} m={compact} id={i}'
                m = take(r'ACF_INPUT '+prefix+rf' x={x} lr=(\[[^]]*\]) weights=\[(\[[^]]*\])\] parameter=(.+)')
                if integers(m[1])!=[lr0,compact] or integers(m[2])!=[weight0,compact]:
                    raise ValueError('input differs from reviewed product coordinates')
                parameter = m[3]
                m = take(r'ACF_MODULE '+prefix+r' ambient=(\[[^]]*\]) target=(\[[^]]*\]) solution=(\[[^]]*\]) ranks=(\[[^]]*\])')
                w,target,solution,ranks = map(integers,m.groups())
                if (len(w)!=ambient or len(target)!=rows or len(solution)!=cols or len(ranks)!=2
                        or product(px,w,rows)!=target or product(q,solution,rows)!=target
                        or ranks!=[sum(a*b for a,b in zip(r,solution)) for r in rank_rows]):
                    raise ValueError('module integral solve, projection or weighted rank mismatch')
                expected = ([compact+1,0] if i<2 else
                            [0,compact+1] if i<4 else [compact+1,compact+1])
                if ranks != expected:
                    raise ValueError('numerical multiplicity differs from tensor expectation')
                modules.append(dict(ambient=w,m=compact,id=i,x=x,parameter=parameter,ranks=ranks,target=target,solution=solution))
        if top != [1,2]:
            raise ValueError('orbit identities require independent remapping')
        w = [m['ambient'] for m in modules[:5]]
        identities = [[a-b for a,b in zip(w[4],w[3])],w[0],
                      [a-b for a,b in zip(w[4],w[1])],w[2]]
        for j,(index,identity) in enumerate(zip(bindings,identities)):
            if (index not in range(cols) or columns[index]['orbit'] != (1 if j<2 else 2)
                    or columns[index]['rank'] != 1 or columns[index]['ambient'] != identity):
                raise ValueError('full Phi identity does not bind the claimed orbit sign')
        take(rf'ACF_END bound={bound}')
        result.append(dict(bound=bound,complex_orbits=complex_orbits,real_orbits=real,
            target_complex=ci,top=top,ambient=ambient,rows=rows,columns=cols,kernel=kdim,boundary=bdim,
            exact_Q_rank=qr,full_kernel_verified=True,phi_bindings=bindings,
            tensor_multiplicities_verified=True,
            inducing_dimensions=sorted(set(c['rank'] for c in columns)),
            rank_rows=rank_rows,modules=modules))
    if next(lines,None) is not None:
        raise ValueError('extra product Phi record')
    return result
