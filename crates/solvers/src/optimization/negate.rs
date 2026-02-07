use twine_core::OptimizationProblem;

/// Adapter that negates the objective value.
///
/// Used to implement maximization by minimizing the negated objective.
pub struct NegateObjective<P>(pub P);

impl<const N: usize, P> OptimizationProblem<N> for NegateObjective<P>
where
    P: OptimizationProblem<N>,
{
    type Input = P::Input;
    type Output = P::Output;
    type Error = P::Error;

    fn input(&self, x: &[f64; N]) -> Result<Self::Input, Self::Error> {
        self.0.input(x)
    }

    fn objective(&self, input: &Self::Input, output: &Self::Output) -> Result<f64, Self::Error> {
        self.0.objective(input, output).map(|v| -v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::convert::Infallible;

    use approx::assert_relative_eq;

    struct TestProblem;

    impl OptimizationProblem<1> for TestProblem {
        type Input = f64;
        type Output = f64;
        type Error = Infallible;

        fn input(&self, x: &[f64; 1]) -> Result<f64, Self::Error> {
            Ok(x[0])
        }

        fn objective(&self, _input: &f64, output: &f64) -> Result<f64, Self::Error> {
            Ok(*output)
        }
    }

    #[test]
    fn negate_objective_flips_sign() {
        let problem = TestProblem;
        let input = 1.0;
        let output = -2.0;

        let original_obj = problem.objective(&input, &output).unwrap();

        let negated = NegateObjective(problem);
        let negated_obj = negated.objective(&input, &output).unwrap();

        assert_relative_eq!(original_obj, -2.0);
        assert_relative_eq!(negated_obj, 2.0);
    }
}
