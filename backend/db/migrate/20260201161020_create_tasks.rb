class CreateTasks < ActiveRecord::Migration[8.1]
  def change
    create_table :tasks do |t|
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

    add_index :tasks, [:uid, :status]
    add_index :tasks, [:uid, :updated_at]
    add_index :tasks, [:uid, :deleted_at]
  end
end
