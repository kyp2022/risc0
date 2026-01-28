
# Python环境使用说明

## 环境信息
- Python版本: Python 3.14.2
- pip版本: pip 25.3
- 虚拟环境名称: python_env

## 如何使用

### 1. 激活虚拟环境
```bash
source python_env/bin/activate
```

激活后，你会看到命令行提示符前面有 `(python_env)` 标志。

### 2. 退出虚拟环境
```bash
deactivate
```

### 3. 安装Python包
在激活虚拟环境后，使用pip安装包：
```bash
pip install 包名
```

### 4. 查看已安装的包
```bash
pip list
```

### 5. 导出依赖列表
```bash
pip freeze > requirements.txt
```

### 6. 从requirements.txt安装依赖
```bash
pip install -r requirements.txt
```

## 最佳实践
1. 始终在虚拟环境中进行Python开发
2. 为每个项目创建独立的虚拟环境
3. 使用requirements.txt管理项目依赖
4. 定期更新pip: `pip install --upgrade pip`

## 其他有用的命令
- 查看Python位置: `which python`
- 查看pip位置: `which pip`
- 查看Python路径: `python -c "import sys; print(sys.path)"`
