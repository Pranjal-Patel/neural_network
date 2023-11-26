use std::fs;
use std::io::{stdout, Write};
use std::marker::PhantomData;

use crate::activations::*;
use crate::matrix::Matrix;

use nanoserde::{DeJson, SerJson};

#[derive(SerJson, DeJson)]
struct Data {
    layers: Vec<usize>,
    learning_rate: f64,
    weights: Vec<Matrix>,
    biases: Vec<Matrix>,
}

impl<F: ActivationFunc> From<&Network<F>> for Data {
    fn from(value: &Network<F>) -> Self {
        Self {
            layers: value.layers.clone(),
            learning_rate: value.learning_rate,
            weights: value.weights.clone(),
            biases: value.biases.clone(),
        }
    }
}

pub struct Network<F: ActivationFunc> {
    /// Size of each layers
    layers: Vec<usize>,

    /// Vector of matrices representing the
    /// weights of each connection
    weights: Vec<Matrix>,

    /// Vector of matrices representing the
    /// biases of neurons
    biases: Vec<Matrix>,

    /// Vector of matrices representing the
    /// actual value of each neuron
    data: Vec<Matrix>,

    /// Learning rate of this nerual network
    learning_rate: f64,

    _activation: PhantomData<F>,
}

impl<F: ActivationFunc> Network<F> {
    pub fn new(layers: Vec<usize>, learning_rate: f64) -> Self {
        let mut weights = Vec::with_capacity(layers.len() - 1);
        let mut biases = Vec::with_capacity(layers.len() - 1);

        for i in 0..layers.len() - 1 {
            weights.push(Matrix::random(layers[i + 1], layers[i]));
            biases.push(Matrix::random(layers[i + 1], 1));
        }

        Self {
            layers,
            weights,
            biases,
            learning_rate,
            data: vec![],
            _activation: PhantomData,
        }
    }

    pub fn load(file_path: &str) -> std::io::Result<Self> {
        let contents = fs::read_to_string(file_path)?;

        let Ok(data) = Data::deserialize_json(&contents) else {
            panic!("Failed to parse data");
        };

        Ok(Self {
            layers: data.layers,
            weights: data.weights,
            biases: data.biases,
            learning_rate: data.learning_rate,
            data: Vec::new(),
            _activation: PhantomData,
        })
    }

    pub fn save(&self, file_path: &str) -> std::io::Result<()> {
        let mut file = fs::File::create(file_path)?;

        let data = Data::from(self);
        file.write_all(data.serialize_json().as_bytes())?;

        Ok(())
    }

    pub fn feed_forward(&mut self, inputs: Vec<f64>) -> Vec<f64> {
        if inputs.len() != self.layers[0] {
            panic!("Invalid number of inputs");
        }

        let mut current = Matrix::row(inputs).transpose();
        self.data = vec![current.clone()];

        for i in 0..self.layers.len() - 1 {
            current = self.weights[i]
                .mul(&current)
                .add(&self.biases[i])
                .map(F::function);

            self.data.push(current.clone());
        }

        current.transpose()[0].to_owned()
    }

    pub fn back_propogate(&mut self, outputs: Vec<f64>, targets: Vec<f64>) {
        if targets.len() != *self.layers.last().unwrap() {
            panic!("Invalid number of targets");
        }

        let parsed = Matrix::row(outputs);
        let mut errors = Matrix::row(targets).sub(&parsed);
        let mut gradients = parsed.map(F::derivative);

        for i in (0..self.layers.len() - 1).rev() {
            gradients = gradients.dot(&errors).map(|x| x * self.learning_rate);

            self.weights[i] = self.weights[i].add(&gradients.mul(&self.data[i].transpose()));
            self.biases[i] = self.biases[i].add(&gradients);

            errors = self.weights[i].transpose().mul(&errors);
            gradients = self.data[i].map(F::derivative);
        }
    }

    pub fn train(&mut self, inputs: Vec<Vec<f64>>, targets: Vec<Vec<f64>>, epochs: u16) {
        for i in 1..=epochs {
            if epochs < 100 || i % (epochs / 100) == 0 {
                print!("\r[Log] Epoch {i} of {epochs}");
                stdout().flush().unwrap();
            }

            for j in 0..inputs.len() {
                let outputs = self.feed_forward(inputs[j].clone());
                self.back_propogate(outputs, targets[j].clone());
            }
        }

        println!("\r[Log] Done training!                  ");
    }
}
