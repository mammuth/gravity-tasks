class ChangeTasksIdToString < ActiveRecord::Migration[8.1]
  def up
    create_table :tasks_new, id: false do |t|
      t.string :id, primary_key: true
      t.string :uid, null: false
      t.string :title, null: false
      t.string :status, null: false, default: "active"
      t.float :position, null: false
      t.datetime :done_at
      t.datetime :archived_at
      t.datetime :deleted_at
      t.integer :revision, null: false, default: 0
      t.datetime :created_at, null: false
      t.datetime :updated_at, null: false
    end

    execute <<~SQL
      INSERT INTO tasks_new (id, uid, title, status, position, done_at, archived_at, deleted_at, revision, created_at, updated_at)
      SELECT CAST(id AS TEXT), uid, title, status, position, done_at, archived_at, deleted_at, revision, created_at, updated_at
      FROM tasks
    SQL

    drop_table :tasks
    rename_table :tasks_new, :tasks

    add_index :tasks, [:uid, :status]
    add_index :tasks, [:uid, :updated_at]
    add_index :tasks, [:uid, :deleted_at]
  end

  def down
    create_table :tasks_new do |t|
      t.string :uid, null: false
      t.string :title, null: false
      t.string :status, null: false, default: "active"
      t.float :position, null: false
      t.datetime :done_at
      t.datetime :archived_at
      t.datetime :deleted_at
      t.integer :revision, null: false, default: 0
      t.timestamps
    end

    execute <<~SQL
      INSERT INTO tasks_new (uid, title, status, position, done_at, archived_at, deleted_at, revision, created_at, updated_at)
      SELECT uid, title, status, position, done_at, archived_at, deleted_at, revision, created_at, updated_at
      FROM tasks
    SQL

    drop_table :tasks
    rename_table :tasks_new, :tasks

    add_index :tasks, [:uid, :status]
    add_index :tasks, [:uid, :updated_at]
    add_index :tasks, [:uid, :deleted_at]
  end
end
