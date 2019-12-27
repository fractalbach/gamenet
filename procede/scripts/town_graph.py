import argparse
import json
from pathlib import Path

import matplotlib.pyplot as plt
import networkx as nx


class Node:
    def __init__(self, d) -> None:
        self.uv = d['uv']
        self.i = d['i']

    def add(self, graph: nx.Graph) -> None:
        graph.add_node(self.i, pos=(self.uv['x'], self.uv['y']))


class Edge:
    def __init__(self, d) -> None:
        self.a = d['a']
        self.b = d['b']

    def add(self, graph: nx.Graph) -> None:
        graph.add_edge(self.a, self.b)


def graph_town(path: Path):
    with path.open() as f:
        data = json.load(f)

    graph = nx.Graph()

    nodes = data['nodes']['elements']
    for node_data, _rect in nodes.values():
        node = Node(node_data)
        node.add(graph)

    edges = data['edges']['elements']
    for edge_dict, _rect in edges.values():
        edge = Edge(edge_dict)
        edge.add(graph)

    nx.draw(
        graph,
        pos=nx.get_node_attributes(graph, 'pos'),
        node_size=2,
    )
    plt.axis('equal')
    plt.show()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('path', type=Path)
    args = parser.parse_args()
    graph_town(args.path)


if __name__ == '__main__':
    main()
