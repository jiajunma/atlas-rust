import unittest
from math_cycle_check import check_product_phi, exact_rank, product, mathematical_evidence


def sample():
    lines=[]
    for bound in (4,6):
        p=f'bound={bound}'
        lines.append(f'ACF_BEGIN {p} complex=4 real=3')
        for i,(h,d,dim,forms) in enumerate((
            ([0,0],[0,0],0,[0]),([0,1],[0,2],2,[]),
            ([1,0],[2,0],2,[1,2]),([1,1],[2,2],4,[]))):
            lines.append(f'ACF_COMPLEX {p} orbit={i} H={h} diagram={d} dimension={dim} real_forms={forms}')
        for i,(h,x,c,dim,count) in enumerate((([0,0],0,0,0,1),([1,0],0,2,2,3),([1,0],1,2,2,3))):
            lines.append(f'ACF_REAL {p} orbit={i} H={h} x={x} complex={c} dimension={dim} pairs={count}')
        lines.append(f'ACF_MATRIX {p} complex=2 closure=[0,1,2] top=[1,2] ambient=7 rows=6 columns=7 kernel=1 boundary=1')
        identity=[[int(i==j) for i in range(7)] for j in range(7)]
        px=[c[1:] for c in identity]
        for j,col in enumerate(px):
            lines.append(f'ACF_PX {p} column={j} vector={col}')
        lines.append(f'ACF_BOUNDARY {p} column=0 vector={identity[0]}')
        for j,a in enumerate(identity):
            oi,local,rank=(0,0,1) if j==0 else ((1 if j<4 else 2),(j-1)%3,(j-1)%3+1)
            lines.append(f'ACF_COLUMN {p} column={j} orbit={oi} pair={local} rank={rank} ambient={a} quotient={px[j]}')
            lines.append(f'ACF_PHI {p} column={j} inducing=synthetic')
        lines.append(f'ACF_KERNEL {p} column=0 vector={identity[0]}')
        rank_rows=[[0,1,2,3,0,0,0],[0,0,0,0,1,2,3]]
        for oi,row in zip((1,2),rank_rows):
            lines.append(f'ACF_RANK {p} orbit={oi} row={row} kernel_product=[0]')
        for j,index in enumerate((1,1,4,4)):
            lines.append(f'ACF_BIND {p} identity={j} column={index}')
        for compact in range(3):
            for i,(x,lr0,weight0) in enumerate(((0,-1,1),(0,0,2),(1,-1,-1),(1,0,-2),(2,0,0))):
                prefix=f'{p} m={compact} id={i}'
                w=[0]*7
                if i in (0,1,4): w[1+compact]=1
                if i in (2,3,4): w[4+compact]=1
                ranks=[sum(a*b for a,b in zip(row,w)) for row in rank_rows]
                lines.append(f'ACF_INPUT {prefix} x={x} lr={[lr0,compact]} weights={[[weight0,compact]]} parameter=synthetic')
                lines.append(f'ACF_MODULE {prefix} ambient={w} target={w[1:]} solution={w} ranks={ranks}')
        lines.append(f'ACF_END {p}')
    return '\n'.join(lines)+'\n'


class ProductPhiTests(unittest.TestCase):
    def test_checker_is_enforced_in_driver_and_review(self):
        from math_suite import compare
        from math_suite_review import independently_classify
        case = {"id": "product", "expected": "accept",
                "mathematical_check": "sl2_su2_numerical_cycles"}
        good = {"exit_status": 0, "timed_out": False}
        raw = ("MATH_BEGIN product\n" + sample() + "MATH_END product\n").encode()
        bad = raw.replace(b"identity=0 column=1", b"identity=0 column=4", 1)
        entries = {e: good for e in ("oracle", "rust")}
        for engine, expected in (("oracle", "ORACLE_INVARIANT_FAILURE"),
                                 ("rust", "RUST_INVARIANT_FAILURE")):
            streams = {e: (bad if e == engine else raw, b"") for e in entries}
            self.assertEqual(compare(case, entries, streams)["status"], expected)
            self.assertEqual(independently_classify(case, entries, streams), expected)
        failed = mathematical_evidence(case, dict(good, exit_status=1), raw, b"error")
        self.assertEqual(failed["status"], "NOT_RUN_EXECUTION_FAILED")

    def test_complete_discovery_and_nontrivial_inducing_dimensions(self):
        result=check_product_phi(sample())
        self.assertEqual([r['bound'] for r in result],[4,6])
        for r in result:
            self.assertEqual((r['columns'],r['kernel'],r['exact_Q_rank']),(7,1,6))
            self.assertEqual(r['inducing_dimensions'],[1,2,3])
            self.assertEqual(len(r['modules']),15)
            self.assertEqual(r['modules'][-1]['ranks'],[3,3])

    def test_component_and_column_identity(self):
        for old,new in (('complex=2 closure=','complex=1 closure='),
                        ('column=1 orbit=1','column=1 orbit=2'),
                        ('diagram=[2, 0]','diagram=[0, 2]'),
                        ('lr=[-1, 0]','lr=[0, 0]'),
                        ('weights=[[1, 0]]','weights=[[-1, 0]]')):
            with self.subTest(old=old),self.assertRaises(ValueError):
                check_product_phi(sample().replace(old,new,1))

    def test_missing_extra_and_reordered_records(self):
        text=sample()
        for changed in (text.replace('ACF_END bound=6\n',''),text+'ACF_EXTRA\n',
                        text.replace('ACF_PHI bound=4 column=0 inducing=synthetic\n',''),
                        text.replace('ACF_PX bound=4 column=0','ACF_PX bound=4 column=1',1)):
            with self.assertRaises(ValueError):
                check_product_phi(changed)

    def test_entire_kernel_and_dimension_weights(self):
        for old,new in (
            ('ACF_KERNEL bound=4 column=0 vector=[1, 0, 0, 0, 0, 0, 0]',
             'ACF_KERNEL bound=4 column=0 vector=[0, 0, 0, 0, 0, 0, 0]'),
            ('ACF_KERNEL bound=4 column=0 vector=[1, 0, 0, 0, 0, 0, 0]',
             'ACF_KERNEL bound=4 column=0 vector=[0, 1, 0, 0, 0, 0, 0]'),
            ('row=[0, 1, 2, 3, 0, 0, 0]','row=[0, 1, 1, 1, 0, 0, 0]'),
            ('kernel_product=[0]','kernel_product=[1]'),
            ('rank=2 ambient=','rank=1 ambient=')):
            with self.subTest(old=old),self.assertRaises(ValueError):
                check_product_phi(sample().replace(old,new,1))

    def test_boundary_projection_and_module_residual(self):
        for old,new in (
            ('ACF_BOUNDARY bound=4 column=0 vector=[1, 0, 0, 0, 0, 0, 0]',
             'ACF_BOUNDARY bound=4 column=0 vector=[0, 1, 0, 0, 0, 0, 0]'),
            ('solution=[0, 1, 0, 0, 0, 0, 0]','solution=[0, 0, 0, 0, 0, 0, 0]'),
            ('ranks=[2, 0]','ranks=[1, 0]')):
            with self.subTest(old=old),self.assertRaises(ValueError):
                check_product_phi(sample().replace(old,new,1))

    def test_binding_is_not_fitted_to_rank(self):
        with self.assertRaisesRegex(ValueError, 'Phi identity'):
            check_product_phi(sample().replace('identity=0 column=1','identity=0 column=4',1))

    def test_independent_tensor_expectation_not_just_solve_consistency(self):
        text=sample()
        # Change inducing dimensions, all weighted rows and all outputs together.
        # Exact linear algebra still agrees; the independent tensor rule must fail.
        text=text.replace('rank=2 ambient=', 'rank=4 ambient=')
        text=text.replace('row=[0, 1, 2, 3, 0, 0, 0]', 'row=[0, 1, 4, 3, 0, 0, 0]')
        text=text.replace('row=[0, 0, 0, 0, 1, 2, 3]', 'row=[0, 0, 0, 0, 1, 4, 3]')
        for old,new in (('ranks=[2, 0]','ranks=[4, 0]'),('ranks=[0, 2]','ranks=[0, 4]'),
                        ('ranks=[2, 2]','ranks=[4, 4]')):
            text=text.replace(old,new)
        with self.assertRaisesRegex(ValueError, 'tensor expectation'):
            check_product_phi(text)

    def test_exact_linear_algebra_helpers(self):
        self.assertEqual(exact_rank([[2,4],[3,6]],2),1)
        self.assertEqual(exact_rank([[2,4],[3,7]],2),2)
        self.assertEqual(product([[2,4],[3,7]],[2,-1],2),[1,1])
        self.assertEqual(exact_rank([],0),0)
        with self.assertRaises(ValueError):
            product([[1]],[],1)
        with self.assertRaises(ValueError):
            exact_rank([[1,2]],1)


if __name__=='__main__':
    unittest.main()
